use core_lib::{ClientData, ClientId};
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::gtk::pango;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};

#[derive(Debug)]
pub struct Clients {
    active: bool,
    data: ClientData,
    id: ClientId,
    scale: f64,
}

#[derive(Debug)]
pub enum ClientsInput {
    SetActive(bool),
}

#[derive(Debug)]
pub struct ClientsInit {
    data: ClientData,
    id: ClientId,
    scale: f64,
}

#[derive(Debug)]
pub enum ClientsOutput {}

#[relm4::factory(pub)]
impl FactoryComponent for Clients {
    type Init = ClientsInit;
    type Input = ClientsInput;
    type Output = ClientsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::FlowBoxChild {
            gtk::Button {
                #[watch]
                set_css_classes: if self.active { &["active", "client", "no-hover"] } else { &["client", "no-hover"] },
                set_cursor_from_name: Some("pointer"),
                gtk::Frame {
                    #[wrap(Some)]
                    set_label_widget = &gtk::Label {
                        set_overflow: gtk::Overflow::Visible,
                        set_margin_start: 6,
                        set_ellipsize: pango::EllipsizeMode::End,
                        set_label: if self.data.title.trim().is_empty() {
                            &self.data.class
                        } else {
                            &self.data.title
                        },
                    },
                    set_label_align: 0.5,
                    gtk::Image {
                        set_css_classes: &["client-image"],
                        set_pixel_size: (f64::from(scale(self.data.height, self.scale).min(scale(self.data.width, self.scale)).clamp(50, 600)) / 1.6) as i32 - 20,
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            active: false,
            data: init.data,
            id: init.id,
            scale: init.scale,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            ClientsInput::SetActive(active) => {
                self.active = active;
            }
        }
    }
}

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}
