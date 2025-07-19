use crate::{ClientId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransferType {
    /// send from the keybind to open the overview
    OpenOverview,
    /// send from the keybind to open the switch
    OpenSwitch(OpenSwitch),
    /// send from the keybinds like arrow keys or tab on overview
    SwitchOverview(SwitchOverviewConfig),
    /// send from the keybinds like arrow keys or tab on switch
    SwitchSwitch(SwitchSwitchConfig),
    CloseOverview(CloseOverviewConfig),
    CloseSwitch,
    /// send from the gui itself when typing the launcher
    Type(String),
    /// send from pressing ESC
    Exit,
    /// send from the app itself when new monitor / config changes detected
    Restart,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSwitch {
    pub reverse: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchOverviewConfig {
    pub direction: Direction,
    pub workspace: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchSwitchConfig {
    pub reverse: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CloseOverviewConfig {
    LauncherClick(Identifier),
    LauncherPress(char),
    Windows(WindowsOverride),
    None,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PluginNames {
    Applications,
    Shell,
    Terminal,
    WebSearch,
    Calc,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub plugin: PluginNames,
    // identifies the box in the launcher results
    pub data: Option<Box<str>>,
    // additional data used to get suboption in submenu (only available when launched through click)
    pub data_additional: Option<Box<str>>,
}

impl Identifier {
    pub fn plugin(plugin: PluginNames) -> Self {
        Identifier {
            plugin,
            data: None,
            data_additional: None,
        }
    }

    pub fn data(plugin: PluginNames, data: Box<str>) -> Self {
        Identifier {
            plugin,
            data: Some(data),
            data_additional: None,
        }
    }

    pub fn data_additional(plugin: PluginNames, data: Box<str>, data_additional: Box<str>) -> Self {
        Identifier {
            plugin,
            data: Some(data),
            data_additional: Some(data_additional),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowsOverride {
    ClientId(ClientId),
    WorkspaceID(WorkspaceId),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}
