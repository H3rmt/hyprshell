use crate::shared::workspace_clients::{
    WorkspaceClients, WorkspaceClientsInit, WorkspaceClientsInput, WorkspaceClientsOutput,
};
use core_lib::{ClientData, ClientId, MonitorData, WorkspaceData, WorkspaceId};
use regex::Regex;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::gtk::gdk;
use relm4::prelude::*;

/// Workspace items component - displays a workspace with its clients positioned inside
#[derive(Debug)]
pub struct Workspaces {
    active: bool,
    pub data: WorkspaceData,
    pub workspace_id: WorkspaceId,
    pub remove_html: Regex,
    pub scale: f64,
    pub monitor_data: MonitorData,
    pub clients: FactoryVecDeque<WorkspaceClients>,
}

#[derive(Debug)]
pub enum WorkspacesInput {
    SetActive(bool),
    SetActiveClient(ClientId),
    UpdateClientThumbnail(ClientId, gdk::Texture),
}

#[derive(Debug)]
pub struct WorkspacesInit {
    pub monitor_data: MonitorData,
    pub data: WorkspaceData,
    pub id: WorkspaceId,
    pub remove_html: Regex,
    pub scale: f64,
    pub clients: Vec<(ClientId, ClientData)>,
    pub live_thumbnails: bool,
}

#[derive(Debug)]
pub enum WorkspacesOutput {
    Clicked(WorkspaceId),
    ClickedC(ClientId),
}

#[relm4::factory(pub)]
impl FactoryComponent for Workspaces {
    type Init = WorkspacesInit;
    type Input = WorkspacesInput;
    type Output = WorkspacesOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::FlowBoxChild {
            gtk::Box {
                #[watch]
                set_css_classes: &workspace_css_classes(self.active, self.workspace_id),
                set_width_request: scale(self.monitor_data.width, self.scale),
                set_height_request: scale(self.monitor_data.height, self.scale),
                gtk::Frame {
                    #[wrap(Some)]
                    set_label_widget = &gtk::Button {
                        set_cursor_from_name: Some("pointer"),
                        set_width_request: scale(self.monitor_data.width, self.scale),
                        connect_clicked[sender, id = self.workspace_id] => move |_| sender.output_sender().emit(WorkspacesOutput::Clicked(id)),
                        gtk::Label {
                            set_label: &self.workspace_label(),
                        }
                    },
                    set_label_align: 0.5,
                    self.clients.widget() -> &gtk::Fixed {
                        set_width_request: scale(self.data.width, self.scale),
                        set_height_request: scale(self.data.height, self.scale),
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut clients: FactoryVecDeque<WorkspaceClients> = FactoryVecDeque::builder()
            .launch(gtk::Fixed::default())
            .forward(sender.output_sender(), |msg| match msg {
                WorkspaceClientsOutput::Clicked(id) => WorkspacesOutput::ClickedC(id),
            });

        // Populate clients - sort by floating status (floating windows on top)
        {
            let mut sorted_clients: Vec<_> = init.clients.iter().collect();
            sorted_clients.sort_by(|(_, a), (_, b)| {
                // prefer smaller windows to be on top (for floating)
                if a.floating && b.floating {
                    (b.width as i32 * b.height as i32).cmp(&(a.width as i32 * a.height as i32))
                } else {
                    a.floating.cmp(&b.floating)
                }
            });

            let mut guard = clients.guard();
            for (id, client) in sorted_clients {
                if client.enabled {
                    guard.push_back(WorkspaceClientsInit {
                        id: *id,
                        scale: init.scale,
                        data: client.clone(),
                        live_thumbnails: init.live_thumbnails,
                    });
                }
            }
        }

        Self {
            active: false,
            data: init.data,
            monitor_data: init.monitor_data,
            workspace_id: init.id,
            remove_html: init.remove_html,
            scale: init.scale,
            clients,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            WorkspacesInput::SetActive(active) => {
                self.active = active;
                for (idx, _) in self.clients.iter().enumerate() {
                    self.clients
                        .send(idx, WorkspaceClientsInput::SetActive(false));
                }
            }
            WorkspacesInput::SetActiveClient(id) => {
                self.active = false;
                for (idx, item) in self.clients.iter().enumerate() {
                    self.clients
                        .send(idx, WorkspaceClientsInput::SetActive(id == item.id));
                }
            }
            WorkspacesInput::UpdateClientThumbnail(client_id, texture) => {
                for (idx, item) in self.clients.iter().enumerate() {
                    if item.id == client_id {
                        self.clients
                            .send(idx, WorkspaceClientsInput::UpdateThumbnail(texture));
                        break;
                    }
                }
            }
        };
    }
}

impl Workspaces {
    fn workspace_label(&self) -> String {
        if self.data.name.trim().is_empty() {
            self.workspace_id.to_string()
        } else {
            self.remove_html
                .replace_all(&self.data.name, "")
                .to_string()
        }
    }

    /// Get the client ID at a specific index
    pub fn get_client_id(&self, idx: usize) -> Option<ClientId> {
        self.clients.get(idx).map(|c| c.id)
    }

    /// Get the number of clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}

fn workspace_css_classes(active: bool, id: WorkspaceId) -> Vec<&'static str> {
    let mut classes = vec!["workspace", "no-hover"];
    if active {
        classes.push("active");
    }
    // Special workspaces have negative IDs
    if id < 0 {
        classes.push("special");
    }
    classes
}
