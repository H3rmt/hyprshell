use core_lib::{Active, ClientId, HyprlandData, MonitorId, WorkspaceId};
use gtk::{ApplicationWindow, Button, FlowBox};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowsOverviewData {
    pub config: WindowsOverviewConfig,
    pub window_list: HashMap<ApplicationWindow, WindowsOverviewMonitorData>,
    pub active: Active,
    pub initial_active: Active,
    pub hypr_data: HyprlandData,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct WindowsOverviewConfig {
    pub items_per_row: u8,
    pub scale: f64,
    pub hide_filtered: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
}

#[derive(Debug)]
pub struct WindowsSwitchData {
    pub config: WindowsSwitchConfig,
    pub window: ApplicationWindow,
    pub main_flow: FlowBox,
    pub workspaces: HashMap<WorkspaceId, Button>,
    pub clients: HashMap<ClientId, Button>,
    pub active: Active,
    pub hypr_data: HyprlandData,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct WindowsSwitchConfig {
    pub items_per_row: u8,
    pub scale: f64,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub show_workspaces: bool,
}

#[derive(Debug)]
pub struct WindowsOverviewMonitorData {
    pub id: MonitorId,
    pub workspaces_flow: FlowBox,
    pub workspaces: HashMap<WorkspaceId, gtk::Box>,
    pub clients: HashMap<ClientId, Button>,
}

impl WindowsOverviewMonitorData {
    pub fn new(id: MonitorId, workspaces_flow: FlowBox) -> Self {
        Self {
            id,
            workspaces_flow,
            workspaces: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}
