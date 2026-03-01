use crate::switch::workspace_clients::{Clients, ClientsInit};
use core_lib::{WorkspaceData, WorkspaceId};
use regex::Regex;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct Items {
    active: bool,
    data: WorkspaceData,
    id: WorkspaceId,
    remove_html: Regex,
    scale: f64,
    clients: FactoryVecDeque<Clients>,
}

#[derive(Debug)]
pub enum ItemsInput {
    SetActive(bool),
    SetClients(Vec<Clients>),
}

#[derive(Debug)]
pub struct ItemsInit {
    pub data: WorkspaceData,
    pub id: WorkspaceId,
    pub remove_html: Regex,
    pub scale: f64,
}

#[derive(Debug)]
pub enum ItemsOutput {}

#[relm4::factory(pub)]
impl FactoryComponent for Items {
    type Init = ItemsInit;
    type Input = ItemsInput;
    type Output = ItemsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::FlowBoxChild {
            gtk::Button {
                #[watch]
                set_css_classes: if self.active { &["active", "workspace", "no-hover"] } else { &["workspace", "no-hover"] },
                set_cursor_from_name: Some("pointer"),
                #[name(frame)]
                gtk::Frame {
                    // set_label: if self.data.name.trim().is_empty() {
                    //     Some(&self.id.to_string())
                    // } else {
                    //     Some(&self.remove_html.replace_all(&self.data.name, "").to_string())
                    // },
                    set_label_align: 0.5,
                    gtk::Fixed {
                        set_width_request: scale(self.data.width, self.scale),
                        set_height_request: scale(self.data.height, self.scale),
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let mut clients = FactoryVecDeque::builder()
            .launch(gtk::Fixed::default())
            .detach();

        Self {
            active: false,
            data: init.data,
            id: init.id,
            remove_html: init.remove_html,
            scale: init.scale,
            clients,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ItemsInput::SetClients(_) => {}
            ItemsInput::SetActive(active) => self.active = active,
        };
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match &message {
            ItemsInput::SetClients(clients) => {
                widgets.frame.set_child(Some(self.clients.widget()));
                let mut lock = self.clients.guard();
                lock.clear();
                for data in clients.into_iter() {
                    lock.push_back(ClientsInit { id: data });
                }

                let mut lock = self.clients.guard();
                lock.clear();
                lock.extend(clients.iter());
            }
            _ => {}
        }
        self.update(message, sender.clone());
        self.update_view(widgets, sender);
    }
}

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}
