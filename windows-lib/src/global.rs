use core_lib::config::Windows;
use core_lib::{Active, ClientId, HyprlandData, MonitorId, WorkspaceId};
use exec_lib::get_initial_active;
use gtk::{ApplicationWindow, Button, FlowBox};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowsGlobal {
    pub workspaces_per_row: u8,
    pub scale: f64,
    pub strip_html_from_workspace_title: bool,
    pub data: RefCell<OverviewGlobalData>,
}

impl WindowsGlobal {
    pub fn new(config: Windows) -> Self {
        Self {
            workspaces_per_row: config.workspaces_per_row,
            scale: config.scale,
            strip_html_from_workspace_title: config.strip_html_from_workspace_title,
            data: RefCell::new(OverviewGlobalData::default()),
        }
    }
}

#[derive(Debug)]
pub struct OverviewGlobalData {
    // pub monitor_list: HashMap<ApplicationWindow, (OverviewGlobalMonitorData, Monitor)>,
    pub monitor_list: HashMap<ApplicationWindow, OverviewGlobalMonitorData>,
    pub active: Active,
    pub hypr_data: HyprlandData,
}

impl Default for OverviewGlobalData {
    fn default() -> Self {
        let active = get_initial_active().expect("Failed to get initial active");
        Self {
            monitor_list: HashMap::new(),
            active,
            hypr_data: HyprlandData::default(),
        }
    }
}

#[derive(Debug)]
pub struct OverviewGlobalMonitorData {
    pub id: MonitorId,
    // pub connector: GString,

    // used to store a ref to the FlowBox containing the workspaces
    pub workspaces_flow: FlowBox,
    // used to store refs to the Overlays over the workspace Frames
    pub workspace_refs: HashMap<WorkspaceId, Button>,
    // used to store refs to the Overlays containing the clients
    pub client_refs: HashMap<ClientId, Button>,
}

impl OverviewGlobalMonitorData {
    pub fn new(
        id: MonitorId,
        // connector: GString,
        workspaces_flow: FlowBox,
        // workspaces_flow_overlay: Overlay,
    ) -> Self {
        Self {
            id,
            // connector,
            workspaces_flow,
            // workspaces_flow_overlay,
            workspace_refs: HashMap::new(),
            client_refs: HashMap::new(),
        }
    }
}
