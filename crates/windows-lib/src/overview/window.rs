use crate::shared::{Workspaces, WorkspacesInit, WorkspacesInput, WorkspacesOutput};
use core_lib::{
    Active, ClientData, ClientId, MonitorData, OVERVIEW_NAMESPACE, WorkspaceData, WorkspaceId,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
use relm4::adw::prelude::*;
use relm4::adw::{gdk, gtk};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::{Orientation, SelectionMode};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use tracing::trace;

#[derive(Debug)]
pub struct OverviewWindow {
    general: config_lib::WindowsGeneral,
    open: bool,
    // gtk
    window: gtk::ApplicationWindow,
    /// Regex for removing HTML tags from strings
    remove_html: Regex,
    /// Factory for workspaces
    items: FactoryVecDeque<Workspaces>,
}

#[derive(Debug)]
pub enum OverviewWindowInput {
    SetGeneral(config_lib::WindowsGeneral),
    OpenOverview((OverviewWindowData, u16)),
    CloseOverview,
    ReloadOverview(OverviewWindowData),
    SetActive(Active, Active),
}

#[derive(Debug)]
pub struct OverviewWindowInit {
    pub general: config_lib::WindowsGeneral,
    pub gtk_monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum OverviewWindowOutput {
    Clicked(WorkspaceId),
    ClickedC(ClientId),
}

#[relm4::component(pub)]
impl SimpleComponent for OverviewWindow {
    type Init = OverviewWindowInit;
    type Input = OverviewWindowInput;
    type Output = OverviewWindowOutput;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_css_classes: &["window"],
            set_default_size: (100, 100),
            #[local_ref]
            itemsw -> gtk::FlowBox {
                set_css_classes: &["monitor"],
                set_selection_mode: SelectionMode::None,
                set_orientation: Orientation::Horizontal,
                #[watch]
                set_max_children_per_line: u32::from(model.general.items_per_row),
                #[watch]
                set_min_children_per_line: u32::from(model.general.items_per_row),
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing OverviewWindow");

        let items: FactoryVecDeque<Workspaces> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.output_sender(), |msg| match msg {
                WorkspacesOutput::Clicked(ws) => OverviewWindowOutput::Clicked(ws),
                WorkspacesOutput::ClickedC(id) => OverviewWindowOutput::ClickedC(id),
            });

        let model = Self {
            general: init.general,
            open: false,
            window: root.clone(),
            remove_html: Regex::new(r"<[^>]*>").expect("invalid regex"),
            items,
        };

        let itemsw: gtk::FlowBox = model.items.widget().clone();

        let widgets = view_output!();

        let window = &root;
        window.init_layer_shell();
        window.set_namespace(Some(OVERVIEW_NAMESPACE));
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_monitor(Some(&init.gtk_monitor));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        trace!("overview::root::window::update: {message:?}");
        match message {
            OverviewWindowInput::SetGeneral(general) => {
                self.general = general;
            }
            OverviewWindowInput::OpenOverview((data, top_offset)) => {
                if !self.open {
                    self.open = true;
                    self.window.set_margin(Edge::Top, top_offset as i32);
                    self.open_overview(data);
                } else {
                    trace!("already open");
                }
            }
            OverviewWindowInput::SetActive(prev, next) => {
                if self.open {
                    self.set_active(prev, next);
                } else {
                    trace!("not open");
                }
            }
            OverviewWindowInput::CloseOverview => {
                if self.open {
                    self.open = false;
                    self.close_overview();
                } else {
                    trace!("not open");
                }
            }
            OverviewWindowInput::ReloadOverview(data) => {
                if self.open {
                    self.reload_overview(data);
                } else {
                    trace!("not open");
                }
            }
        }
    }
}

impl OverviewWindow {
    fn open_overview(&mut self, data: OverviewWindowData) {
        trace!("Showing window {:?}", self.window.id());
        self.window.set_visible(true);
        self.window.grab_focus();

        self.populate_workspace_mode(&data, self.general.scale);
    }

    fn populate_workspace_mode(&mut self, data: &OverviewWindowData, scale: f64) {
        let mut lock = self.items.guard();
        lock.clear();

        for (wid, workspace_data) in &data.workspaces {
            if !workspace_data.any_client_enabled {
                trace!("skipping workspace {} with no enabled clients", wid);
                continue;
            }

            // Get clients for this workspace
            let workspace_clients: Vec<_> = data
                .clients
                .iter()
                .filter(|(_, client)| client.workspace == *wid && client.enabled)
                .map(|(id, data)| (*id, data.clone()))
                .collect();

            lock.push_back(WorkspacesInit {
                monitor_data: data.monitor.clone(),
                remove_html: self.remove_html.clone(),
                id: *wid,
                data: workspace_data.clone(),
                scale,
                clients: workspace_clients,
            });
        }
        drop(lock);

        // Set active workspace
        for (idx, item) in self.items.iter().enumerate() {
            if item.workspace_id == data.active.workspace {
                if let Some(active_client) = data.active.client {
                    self.items
                        .send(idx, WorkspacesInput::SetActiveClient(active_client));
                } else {
                    self.items.send(idx, WorkspacesInput::SetActive(true));
                }
                break;
            }
        }
    }

    fn set_active(&mut self, old_active: Active, new_active: Active) {
        for (idx, item) in self.items.iter().enumerate() {
            if item.workspace_id == old_active.workspace
                && old_active.workspace != new_active.workspace
            {
                self.items.send(idx, WorkspacesInput::SetActive(false));
            }
            if item.workspace_id == new_active.workspace {
                if let Some(cid) = new_active.client {
                    self.items.send(idx, WorkspacesInput::SetActiveClient(cid));
                } else {
                    if old_active.workspace != new_active.workspace {
                        self.items.send(idx, WorkspacesInput::SetActive(true));
                    }
                }
            }
        }
    }

    fn close_overview(&mut self) {
        trace!("Hiding window {:?}", self.window.id());
        self.window.set_visible(false);

        // Clear UI
        {
            let mut lock = self.items.guard();
            lock.clear();
        }
    }

    fn reload_overview(&mut self, data: OverviewWindowData) {
        if data.clients.is_empty() {
            self.close_overview();
            return;
        }
        self.populate_workspace_mode(&data, self.general.scale);
    }
}

#[derive(Debug)]
pub struct OverviewWindowData {
    pub active: Active,
    pub clients: Vec<(ClientId, ClientData)>,
    pub workspaces: Vec<(WorkspaceId, WorkspaceData)>,
    pub monitor: MonitorData,
}
