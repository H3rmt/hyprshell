use crate::data::{SortConfig, collect_data};
use crate::next::{find_next_client, find_next_workspace};
use crate::overview::OverviewRootInput;
use crate::shared::{Workspaces, WorkspacesInit, WorkspacesInput};
use crate::switch::clients::{Clients, ClientsInit};
use core_lib::{Active, ByFirst, Direction, HyprlandData, SWITCH_NAMESPACE};
use exec_lib::switch::{switch_client, switch_workspace};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
use relm4::adw::glib::ControlFlow;
use relm4::adw::gtk;
use relm4::adw::gtk::glib;
use relm4::adw::prelude::*;
use relm4::gtk::gdk::Key;
use relm4::gtk::{EventControllerKey, Orientation, SelectionMode};
use relm4::prelude::*;
use std::time::Duration;
use tracing::{debug, error, trace};

const KILL_TIMEOUT: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub struct SwitchRoot {
    general: config_lib::WindowsGeneral,
    switch: config_lib::Switch,
    open: bool,
    data: SwitchData,
    // gtk
    window: gtk::ApplicationWindow,
    controller: Option<gtk::EventController>,
    /// Regex for removing HTML tags from strings
    remove_html: Regex,
    /// Factory for workspace mode (workspaces)
    items: FactoryVecDeque<Workspaces>,
    /// Factory for non-workspace mode (clients)
    clients_only: FactoryVecDeque<Clients>,
}

#[derive(Debug)]
pub enum SwitchRootInput {
    SetSwitch(config_lib::Switch),
    SetGeneral(config_lib::WindowsGeneral),
    OpenSwitch(Direction),
    Switch(Direction),
    CloseSwitch(bool),
    CloseCurrentItem,
    ReloadSwitch,
}

#[derive(Debug)]
pub struct SwitchRootInit {
    pub general: config_lib::WindowsGeneral,
    pub switch: config_lib::Switch,
}

#[derive(Debug)]
pub enum SwitchRootOutput {}

#[relm4::component(pub)]
impl SimpleComponent for SwitchRoot {
    type Init = SwitchRootInit;
    type Input = SwitchRootInput;
    type Output = SwitchRootOutput;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_css_classes: &["window"],
            set_default_size: (100, 100),
            match model.switch.switch_workspaces {
                true => {
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
                false => {
                    #[local_ref]
                    clients_only_w -> gtk::FlowBox {
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
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing SwitchRoot");

        let items: FactoryVecDeque<Workspaces> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let clients_only: FactoryVecDeque<Clients> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let model = Self {
            general: init.general,
            switch: init.switch,
            open: false,
            window: root.clone(),
            controller: None,
            remove_html: Regex::new(r"<[^>]*>").expect("invalid regex"),
            data: SwitchData::default(),
            items,
            clients_only,
        };

        let itemsw: gtk::FlowBox = model.items.widget().clone();
        let clients_only_w: gtk::FlowBox = model.clients_only.widget().clone();
        let widgets = view_output!();

        let window = &root;
        window.init_layer_shell();
        window.set_namespace(Some(SWITCH_NAMESPACE));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        sender
            .input_sender()
            .emit(SwitchRootInput::SetSwitch(model.switch.clone()));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("switch::root::update: {message:?}");
        match message {
            SwitchRootInput::SetSwitch(switch) => {
                self.switch = switch;
                self.setup_keyboard_controller(&sender);
            }
            SwitchRootInput::SetGeneral(general) => {
                self.general = general;
                self.setup_keyboard_controller(&sender);
            }
            SwitchRootInput::OpenSwitch(direction) => {
                if !self.open {
                    self.open = true;
                    self.open_switch(direction);
                } else {
                    sender
                        .input_sender()
                        .emit(SwitchRootInput::Switch(direction));
                }
            }
            SwitchRootInput::Switch(direction) => {
                if self.open {
                    self.navigate(direction);
                } else {
                    trace!("not open");
                }
            }
            SwitchRootInput::CloseSwitch(do_switch) => {
                if self.open {
                    self.open = false;
                    self.close_switch(do_switch);
                } else {
                    trace!("not open");
                }
            }
            SwitchRootInput::CloseCurrentItem => {
                if self.open {
                    self.close_item();
                } else {
                    trace!("not open");
                }
                sender.input_sender().emit(SwitchRootInput::ReloadSwitch);
            }
            SwitchRootInput::ReloadSwitch => {
                if self.open {
                    self.reload_switch();
                } else {
                    trace!("not open");
                }
            }
        }
    }
}

impl SwitchRoot {
    fn setup_keyboard_controller(&mut self, sender: &ComponentSender<Self>) {
        // TODO add a check in config check so these always succeed
        if let Some(k) = Key::from_name(self.switch.key.to_string()) {
            if let Some(kk) = Key::from_name(self.switch.kill_key.to_string()) {
                let key_controller = EventControllerKey::new();
                let sender_2 = sender.clone();
                key_controller.connect_key_pressed(move |_, key, _, _| {
                    trace!("Key pressed: {:?}", key);
                    handle_key(key, k, kk, sender_2.clone())
                });
                if let Some(controller) = self.controller.take() {
                    self.window.remove_controller(&controller);
                }
                self.window.add_controller(key_controller);
            } else {
                error!("Invalid kill key name: {}", self.switch.kill_key);
            }
        } else {
            error!("Invalid key name: {}", self.switch.key);
        }
    }

    fn open_switch(&mut self, direction: Direction) {
        let (hypr_data, active_prev) = match collect_data(&SortConfig {
            filter_current_monitor: self.switch.filter_by_current_monitor,
            filter_current_workspace: self.switch.filter_by_current_workspace,
            filter_same_class: self.switch.filter_by_same_class,
            sort_recent: true,
            exclude_workspaces: if self.switch.exclude_workspaces.is_empty() {
                None
            } else {
                Some(self.switch.exclude_workspaces.clone())
            },
        }) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to collect data: {}", e);
                return;
            }
        };

        let active = if self.switch.switch_workspaces {
            find_next_workspace(
                &direction,
                true,
                &hypr_data,
                active_prev,
                self.general.items_per_row,
            )
        } else {
            find_next_client(
                &direction,
                true,
                &hypr_data,
                active_prev,
                self.general.items_per_row,
            )
        };
        self.data = SwitchData {
            active,
            hypr_data: hypr_data.clone(),
        };

        trace!("Showing window {:?}", self.window.id());
        self.window.set_visible(true);
        self.window.grab_focus();

        if self.switch.switch_workspaces {
            self.populate_workspace_mode(&hypr_data, self.general.scale, self.data.active);
        } else {
            self.populate_clients_only_mode(&hypr_data, self.general.scale, self.data.active);
        }
    }

    fn populate_workspace_mode(&mut self, hypr_data: &HyprlandData, scale: f64, active: Active) {
        let mut lock = self.items.guard();
        lock.clear();

        for (wid, workspace_data) in &hypr_data.workspaces {
            // Get clients for this workspace
            let workspace_clients: Vec<_> = hypr_data
                .clients
                .iter()
                .filter(|(_, client)| client.workspace == *wid && client.enabled)
                .map(|(id, data)| (*id, data.clone()))
                .collect();

            // Skip workspaces with no enabled clients
            if workspace_clients.is_empty() {
                trace!("skipping workspace {} with no enabled clients", wid);
                continue;
            }
            let Some(monitor) = hypr_data.monitors.find_by_first(&workspace_data.monitor) else {
                error!(
                    "Workspace {} has invalid monitor {}",
                    wid, workspace_data.monitor
                );
                continue;
            };
            lock.push_back(WorkspacesInit {
                monitor_data: monitor.clone(),
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
            if item.workspace_id == active.workspace {
                self.items.send(idx, WorkspacesInput::SetActive(true));
                break;
            }
        }
    }

    fn populate_clients_only_mode(&mut self, hypr_data: &HyprlandData, scale: f64, active: Active) {
        let mut lock = self.clients_only.guard();
        lock.clear();

        for (id, client) in &hypr_data.clients {
            if !client.enabled {
                continue;
            }
            let Some(monitor) = hypr_data.monitors.find_by_first(&client.monitor) else {
                error!("Client {} has invalid monitor {}", id, client.monitor);
                continue;
            };
            lock.push_back(ClientsInit {
                id: *id,
                scale,
                monitor_data: monitor.clone(),
                data: client.clone(),
            });
        }
        drop(lock);

        // Set active client
        if let Some(active_id) = active.client {
            for (idx, item) in self.clients_only.iter().enumerate() {
                if item.id == active_id {
                    self.clients_only
                        .send(idx, crate::switch::clients::ClientsInput::SetActive(true));
                    break;
                }
            }
        }
    }

    fn navigate(&mut self, direction: Direction) {
        let new_active = if self.switch.switch_workspaces {
            find_next_workspace(
                &direction,
                true,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        } else {
            find_next_client(
                &direction,
                true,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        };

        let old_active = self.data.active;
        self.data.active = new_active;

        if self.switch.switch_workspaces {
            self.update_workspace_active(old_active, new_active);
        } else {
            self.update_clients_only_active(old_active, new_active);
        }
    }

    fn update_workspace_active(&mut self, old_active: Active, new_active: Active) {
        // Update workspace active state
        if old_active.workspace != new_active.workspace {
            for (idx, item) in self.items.iter().enumerate() {
                if item.workspace_id == old_active.workspace {
                    self.items.send(idx, WorkspacesInput::SetActive(false));
                }
                if item.workspace_id == new_active.workspace {
                    self.items.send(idx, WorkspacesInput::SetActive(true));
                    if let Some(cid) = new_active.client {
                        self.items.send(idx, WorkspacesInput::SetActiveClient(cid));
                    }
                }
            }
        }
    }

    fn update_clients_only_active(&mut self, old_active: Active, new_active: Active) {
        // Clear old active
        if let Some(old_id) = old_active.client {
            for (idx, item) in self.clients_only.iter().enumerate() {
                if item.id == old_id {
                    self.clients_only
                        .send(idx, crate::switch::clients::ClientsInput::SetActive(false));
                    break;
                }
            }
        }

        // Set new active
        if let Some(new_id) = new_active.client {
            for (idx, item) in self.clients_only.iter().enumerate() {
                if item.id == new_id {
                    self.clients_only
                        .send(idx, crate::switch::clients::ClientsInput::SetActive(true));
                    break;
                }
            }
        }
    }

    fn close_switch(&mut self, do_switch: bool) {
        trace!("Hiding window {:?}", self.window.id());
        self.window.set_visible(false);

        // Clear UI
        {
            let mut lock = self.items.guard();
            lock.clear();
        }
        {
            let mut lock = self.clients_only.guard();
            lock.clear();
        }

        if do_switch {
            if let Some(id) = self.data.active.client {
                debug!(
                    "Switching to client {}",
                    self.data
                        .hypr_data
                        .clients
                        .iter()
                        .find(|(cid, _)| *cid == id)
                        .map_or_else(|| "<Unknown>".to_string(), |(_, c)| c.title.clone())
                );
                // Defer execution to ensure window is hidden first
                glib::idle_add_local(move || {
                    if let Err(e) = switch_client(id) {
                        tracing::warn!("Failed to switch to client {id:?}: {e}");
                    }
                    ControlFlow::Break
                });
            } else {
                let id = self.data.active.workspace;
                debug!(
                    "Switching to workspace {}",
                    self.data
                        .hypr_data
                        .workspaces
                        .iter()
                        .find(|(wid, _)| *wid == id)
                        .map_or_else(|| "<Unknown>".to_string(), |(_, w)| w.name.clone())
                );
                glib::idle_add_local(move || {
                    if let Err(e) = switch_workspace(id) {
                        tracing::warn!("Failed to switch to workspace {id:?}: {e}");
                    }
                    ControlFlow::Break
                });
            }
        }
    }

    fn close_item(&mut self) {
        if self.switch.switch_workspaces {
            self.kill_workspace_clients();
        } else {
            self.kill_active_client();
        }
    }

    fn kill_active_client(&self) {
        if let Some(id) = self.data.active.client {
            if let Err(e) = exec_lib::kill::kill_client_blocking(id, KILL_TIMEOUT) {
                // TODO: close on killed to let user close window themself
                tracing::warn!("Failed to kill client {id}: {e}");
            }
        }
    }

    fn kill_workspace_clients(&self) {
        let workspace_id = self.data.active.workspace;
        debug!(
            "Killing all clients in workspace {}",
            self.data
                .hypr_data
                .workspaces
                .iter()
                .find(|(wid, _)| *wid == workspace_id)
                .map_or_else(|| workspace_id.to_string(), |(_, w)| w.name.clone())
        );

        let clients_to_kill: Vec<_> = self
            .data
            .hypr_data
            .clients
            .iter()
            .filter(|(_, client)| client.workspace == workspace_id)
            .map(|(id, _)| *id)
            .collect();

        for client_id in clients_to_kill {
            if let Err(e) = exec_lib::kill::kill_client_blocking(client_id, KILL_TIMEOUT) {
                // TODO: close on killed to let user close window themself
                tracing::warn!("Failed to kill client {client_id}: {e}");
            }
        }
    }

    fn reload_switch(&mut self) {
        let (hypr_data, _) = match collect_data(&SortConfig {
            filter_current_monitor: self.switch.filter_by_current_monitor,
            filter_current_workspace: self.switch.filter_by_current_workspace,
            filter_same_class: self.switch.filter_by_same_class,
            sort_recent: true,
            exclude_workspaces: if self.switch.exclude_workspaces.is_empty() {
                None
            } else {
                Some(self.switch.exclude_workspaces.clone())
            },
        }) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to collect data: {}", e);
                return;
            }
        };

        while match self.data.active {
            Active {
                client: Some(id), ..
            } => hypr_data.clients.find_by_first(&id).is_none(),
            Active { workspace: id, .. } => hypr_data.workspaces.find_by_first(&id).is_none(),
        } {
            self.data.active = if self.switch.switch_workspaces {
                find_next_workspace(
                    &Direction::Right,
                    true,
                    &hypr_data,
                    self.data.active,
                    self.general.items_per_row,
                )
            } else {
                find_next_client(
                    &Direction::Right,
                    true,
                    &hypr_data,
                    self.data.active,
                    self.general.items_per_row,
                )
            };
        }

        self.data = SwitchData {
            active: self.data.active,
            hypr_data: hypr_data.clone(),
        };

        if self.switch.switch_workspaces {
            self.populate_workspace_mode(&hypr_data, self.general.scale, self.data.active);
        } else {
            self.populate_clients_only_mode(&hypr_data, self.general.scale, self.data.active);
        }
    }
}

fn handle_key(
    key: Key,
    s_key: Key,
    kill_key: Key,
    event_sender: ComponentSender<SwitchRoot>,
) -> glib::Propagation {
    match key {
        Key::Escape => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::CloseSwitch(false));
            glib::Propagation::Stop
        }
        k if k == s_key || k == Key::l || k == Key::Right => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(Direction::Right));
            glib::Propagation::Stop
        }
        Key::ISO_Left_Tab | Key::grave | Key::dead_grave | Key::h | Key::Left => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(Direction::Left));
            glib::Propagation::Stop
        }
        Key::j | Key::Down => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(Direction::Down));
            glib::Propagation::Stop
        }
        Key::k | Key::Up => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(Direction::Up));
            glib::Propagation::Stop
        }
        k if k == kill_key || k == Key::Delete => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::CloseCurrentItem);
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    }
}

#[derive(Debug)]
pub struct SwitchData {
    pub active: Active,
    pub hypr_data: HyprlandData,
}

impl Default for SwitchData {
    fn default() -> Self {
        Self {
            active: Active {
                client: None,
                workspace: -1,
                monitor: -1,
            },
            hypr_data: HyprlandData::default(),
        }
    }
}
