use crate::{ClientId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferType {
    /// send from the keybind to open the overview
    OpenOverview,
    /// send from the keybind to open the switch
    OpenSwitch(OpenSwitch),
    /// send from the keybinds like arrow keys or tab on overview
    SwitchOverview(SwitchOverviewConfig),
    /// send from the keybinds like arrow keys or tab on switch
    SwitchSwitch(SwitchSwitchConfig),
    /// send from the gui itself when closing the overview
    CloseOverview(CloseOverviewConfig),
    /// send from the gui itself when closing the switch
    CloseSwitch,
    /// send from the gui itself when closing a client (Blocking)
    CloseClientSwitch,
    /// send from the gui itself when closing a client (Blocking)
    CloseClientOverview,
    /// send from the gui itself when typing the launcher
    Type(String),
    /// send from pressing ESC or repressing openOverview
    CloseAll,
    /// send from the app itself when new monitor / config changes detected
    Restart,
    /// send from the app itself, 500ms after starting
    SetActive,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldMod {
    Alt,
    Ctrl,
    Super,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenSwitch {
    pub reverse: bool,
    #[serde(default)]
    pub profile: usize,
    #[serde(default)]
    pub hold_mods: Vec<HoldMod>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwitchOverviewConfig {
    pub direction: Direction,
    pub workspace: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwitchSwitchConfig {
    pub direction: Direction,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloseOverviewConfig {
    LauncherClick(Identifier),
    LauncherPress(char),
    Windows(WindowsOverride),
    None,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PluginNames {
    Applications,
    Shell,
    Terminal,
    WebSearch,
    Calc,
    Path,
    Actions,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub plugin: PluginNames,
    // identifies the box in the launcher results
    pub data: Option<Box<str>>,
    // additional data used to get suboption in submenu (only available when launched through click)
    pub data_additional: Option<Box<str>>,
}

impl Identifier {
    #[must_use]
    pub const fn plugin(plugin: PluginNames) -> Self {
        Self {
            plugin,
            data: None,
            data_additional: None,
        }
    }

    #[must_use]
    pub const fn data(plugin: PluginNames, data: Box<str>) -> Self {
        Self {
            plugin,
            data: Some(data),
            data_additional: None,
        }
    }

    #[must_use]
    pub const fn data_additional(
        plugin: PluginNames,
        data: Box<str>,
        data_additional: Box<str>,
    ) -> Self {
        Self {
            plugin,
            data: Some(data),
            data_additional: Some(data_additional),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
