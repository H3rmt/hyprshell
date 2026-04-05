use crate::data::{SortConfig, collect_data};
use crate::next::{find_next_client, find_next_workspace};
use crate::switch::clients::{Clients, ClientsInit};
use crate::switch::workspaces::{Items, ItemsInit, ItemsInput};
use core_lib::{Active, FindByFirst, HyprlandData, SWITCH_NAMESPACE};
use exec_lib::reset_no_follow_mouse;
use exec_lib::switch::{switch_client, switch_workspace};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
use relm4::adw::gtk;
use relm4::adw::gtk::glib;
use relm4::adw::prelude::*;
use relm4::gtk::gdk::Key;
use relm4::gtk::{EventControllerKey, Orientation, SelectionMode};
use relm4::prelude::*;
use std::time::Duration;
use tracing::{debug, error, trace};

const KILL_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct SwitchRoot {
    general: config_lib::WindowsGeneral,
    switch: config_lib::Switch,
    // gtk
    window: gtk::ApplicationWindow,
    controller: Option<gtk::EventController>,
    /// Regex for removing HTML tags from strings
    remove_html: Regex,
    data: SwitchData,
    /// Factory for workspace mode (workspaces)
    items: FactoryVecDeque<Items>,
    /// Factory for non-workspace mode (clients)
    clients_only: FactoryVecDeque<Clients>,
}

#[derive(Debug)]
pub enum SwitchRootInput {
    SetSwitch(config_lib::Switch),
    SetGeneral(config_lib::WindowsGeneral),
    OpenSwitch(core_lib::Direction),
    Switch(core_lib::Direction),
    CloseSwitch(bool),
    // CloseItem,
}

#[derive(Debug)]
pub struct SwitchRootInit {
    pub general: config_lib::WindowsGeneral,
    pub switch: config_lib::Switch,
}

#[derive(Debug)]
pub enum SwitchRootOutput {
    Closed,
}

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
            #[watch]
            set_visible: model.data.is_open,
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

        let items: FactoryVecDeque<Items> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let clients_only: FactoryVecDeque<Clients> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let model = Self {
            general: init.general,
            switch: init.switch,
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
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::OnDemand);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("switch::root::update: {message:?}");
        match message {
            SwitchRootInput::SetSwitch(switch) => {
                self.switch = switch;
                // self.setup_keyboard_controller(&sender);
            }
            SwitchRootInput::SetGeneral(general) => {
                self.general = general;
                // self.setup_keyboard_controller(&sender);
            }
            SwitchRootInput::OpenSwitch(direction) => {
                self.open_switch(direction);
            }
            SwitchRootInput::Switch(direction) => {
                self.navigate(direction);
            }
            SwitchRootInput::CloseSwitch(do_switch) => {
                // self.close_switch(do_switch);
                sender.output_sender().emit(SwitchRootOutput::Closed);
            } // SwitchRootInput::CloseItem => {
              // self.close_item();
              // }
        }
    }
}

impl SwitchRoot {
    fn setup_keyboard_controller(&mut self, sender: &ComponentSender<Self>) {
        // TODO add a check in config check so these always succeed
        let s_key = Key::from_name(self.switch.key.to_string()).unwrap();
        let kill_key = Key::from_name(self.switch.kill_key.to_string()).unwrap();
        let key_controller = EventControllerKey::new();
        let sender_2 = sender.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            handle_key(key, s_key, kill_key, sender_2.clone())
        });
        let r#mod = self.switch.modifier;
        let sender_2 = sender.clone();
        key_controller.connect_key_released(move |_, key, _, _| {
            handle_release(key, r#mod, sender_2.clone());
        });
        if let Some(controller) = self.controller.take() {
            self.window.remove_controller(&controller);
        }
        self.window.add_controller(key_controller);
    }

    fn open_switch(&mut self, direction: core_lib::Direction) {
        let (clients_data, active_prev) = match collect_data(&SortConfig {
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
                &clients_data,
                active_prev,
                self.general.items_per_row,
            )
        } else {
            find_next_client(
                &direction,
                true,
                &clients_data,
                active_prev,
                self.general.items_per_row,
            )
        };

        trace!("Showing window {:?}", self.window.id());

        self.data = SwitchData {
            active,
            hypr_data: clients_data.clone(),
            is_open: true,
        };

        if self.switch.switch_workspaces {
            self.populate_workspace_mode(&clients_data, self.general.scale, active);
        } else {
            self.populate_clients_only_mode(&clients_data, self.general.scale, active);
        }
    }

    fn populate_workspace_mode(&mut self, clients_data: &HyprlandData, scale: f64, active: Active) {
        let mut lock = self.items.guard();
        lock.clear();

        for (wid, workspace_data) in &clients_data.workspaces {
            // Get clients for this workspace
            let workspace_clients: Vec<_> = clients_data
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
            let Some(monitor) = clients_data.monitors.find_by_first(&workspace_data.monitor) else {
                error!(
                    "Workspace {} has invalid monitor {}",
                    wid, workspace_data.monitor
                );
                continue;
            };
            lock.push_back(ItemsInit {
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
                self.items.send(idx, ItemsInput::SetActive(true));
                break;
            }
        }
    }

    fn populate_clients_only_mode(
        &mut self,
        clients_data: &HyprlandData,
        scale: f64,
        active: Active,
    ) {
        let mut lock = self.clients_only.guard();
        lock.clear();

        for (id, client) in &clients_data.clients {
            if !client.enabled {
                continue;
            }
            let Some(monitor) = clients_data.monitors.find_by_first(&client.monitor) else {
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

    fn navigate(&mut self, direction: core_lib::Direction) {
        let new_active = if self.switch.switch_workspaces {
            find_next_workspace(
                &direction,
                false,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        } else {
            find_next_client(
                &direction,
                false,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        };

        // TODO add in
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
                    self.items.send(idx, ItemsInput::SetActive(false));
                }
                if item.workspace_id == new_active.workspace {
                    self.items.send(idx, ItemsInput::SetActive(true));
                }
            }
        }

        // Update client active state within workspaces using guard for mutable access
        let mut guard = self.items.guard();
        for item in guard.iter_mut() {
            if item.workspace_id == new_active.workspace {
                item.set_active_client(new_active.client);
            } else if item.workspace_id == old_active.workspace
                && old_active.workspace != new_active.workspace
            {
                item.set_active_client(None);
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
    #[cfg(skip)]
    fn close_switch(&mut self, do_switch: bool) {
        reset_no_follow_mouse().ok();

        // Clear UI
        {
            let mut lock = self.items.guard();
            lock.clear();
        }
        {
            let mut lock = self.clients_only.guard();
            lock.clear();
        }

        trace!("Hiding window {:?}", self.window.id());
        self.data.is_open = false;

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
                    glib::ControlFlow::Break
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
                    glib::ControlFlow::Break
                });
            }
        }
    }
    #[cfg(skip)]
    fn close_item(&mut self) {
        let Some(windows) = &self.windows else { return };
        let Some(switch) = &windows.switch else {
            return;
        };

        if switch.switch_workspaces {
            self.kill_workspace_clients();
        } else {
            self.kill_active_client();
        }
    }
    #[cfg(skip)]
    fn kill_active_client(&self) {
        if let Some(id) = self.data.active.client {
            if let Err(e) = exec_lib::kill::kill_client_blocking(id, KILL_TIMEOUT) {
                tracing::warn!("Failed to kill client {id}: {e}");
            }
        }
    }
    #[cfg(skip)]
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
            .filter(|(_, client)| client.workspace == workspace_id && client.enabled)
            .map(|(id, _)| *id)
            .collect();

        for client_id in clients_to_kill {
            if let Err(e) = exec_lib::kill::kill_client_blocking(client_id, KILL_TIMEOUT) {
                tracing::warn!("Failed to kill client {client_id}: {e}");
            }
        }
    }
}

fn handle_release(
    key: Key,
    modifier: config_lib::Modifier,
    event_sender: ComponentSender<SwitchRoot>,
) {
    if ((key == Key::Alt_L || key == Key::Alt_R) && modifier == config_lib::Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R)
            && modifier == config_lib::Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && modifier == config_lib::Modifier::Super)
    {
        event_sender
            .input_sender()
            .emit(SwitchRootInput::CloseSwitch(true));
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
                .emit(SwitchRootInput::Switch(core_lib::Direction::Right));
            glib::Propagation::Stop
        }
        Key::ISO_Left_Tab | Key::grave | Key::dead_grave | Key::h | Key::Left => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(core_lib::Direction::Left));
            glib::Propagation::Stop
        }
        Key::j | Key::Down => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(core_lib::Direction::Down));
            glib::Propagation::Stop
        }
        Key::k | Key::Up => {
            event_sender
                .input_sender()
                .emit(SwitchRootInput::Switch(core_lib::Direction::Up));
            glib::Propagation::Stop
        }
        // k if k == kill_key || k == Key::Delete => {
        //     event_sender.input_sender().emit(SwitchRootInput::CloseItem);
        //     glib::Propagation::Stop
        // }
        _ => glib::Propagation::Proceed,
    }
}

#[derive(Debug)]
pub struct SwitchData {
    pub active: Active,
    pub hypr_data: HyprlandData,
    pub is_open: bool,
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
            is_open: false,
        }
    }
}
