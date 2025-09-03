pub type WorkspaceId = i32;
pub type MonitorId = i128;
pub type ClientId = u64;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Active {
    pub client: Option<ClientId>,
    pub workspace: WorkspaceId,
    pub monitor: MonitorId,
}

#[derive(Debug, Clone)]
pub struct MonitorData {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub connector: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub monitor: MonitorId,
    pub any_client_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ClientData {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub class: String,
    pub title: String,
    pub workspace: WorkspaceId,
    pub monitor: MonitorId,
    pub focus_history_id: i8,
    pub floating: bool,
    pub enabled: bool,
    pub pid: i32,
}

#[derive(Debug, Default)]
pub struct HyprlandData {
    pub clients: Vec<(ClientId, ClientData)>,
    pub workspaces: Vec<(WorkspaceId, WorkspaceData)>,
    pub monitors: Vec<(MonitorId, MonitorData)>,
}

pub trait FindByFirst<ID, Data> {
    fn find_by_first(&self, id: &ID) -> Option<&Data>;
}

impl FindByFirst<ClientId, ClientData> for [(ClientId, ClientData)] {
    fn find_by_first(&self, id: &ClientId) -> Option<&ClientData> {
        self.iter().find(|(addr, _)| *addr == *id).map(|(_, cd)| cd)
    }
}

impl FindByFirst<WorkspaceId, WorkspaceData> for [(WorkspaceId, WorkspaceData)] {
    fn find_by_first(&self, id: &WorkspaceId) -> Option<&WorkspaceData> {
        self.iter().find(|(wid, _)| *wid == *id).map(|(_, wd)| wd)
    }
}

impl FindByFirst<MonitorId, MonitorData> for [(MonitorId, MonitorData)] {
    fn find_by_first(&self, id: &MonitorId) -> Option<&MonitorData> {
        self.iter().find(|(mid, _)| *mid == *id).map(|(_, md)| md)
    }
}
