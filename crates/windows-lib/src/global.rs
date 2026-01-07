use core_lib::{Active, ClientId, HyprlandData, MonitorId, WorkspaceId};
use core_lib::transfer::HoldMod;
use relm4::adw::gtk::{ApplicationWindow, Button, FlowBox};
use relm4::gtk;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub exclude_workspaces: Option<Box<str>>,
}

#[derive(Debug)]
pub struct WindowsSwitchData {
    pub config: WindowsSwitchConfig,
    pub active_hold_mods: Rc<RefCell<Vec<HoldMod>>>,
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
    pub switch_workspaces: bool,
    pub exclude_workspaces: Option<Box<str>>,
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
