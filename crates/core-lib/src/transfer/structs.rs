use crate::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExternalTransferType {
    /// send from the keybind to open the overview
    OpenOverview,
    /// send from the keybind to open the switch
    OpenSwitch(OpenSwitch),
    /// send from releasing the mod key
    CloseSwitch(CloseSwitch),
    /// send from the app itself when new monitor / config / css / hyprconfig changes detected
    Reload,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenSwitch {
    pub reverse: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloseSwitch {
    pub switch: bool,
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
pub enum PluginName {
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
    pub plugin: PluginName,
    // identifies the box in the launcher results
    pub data: Option<Box<str>>,
    // additional data used to get suboption in submenu (only available when launched through click)
    pub data_additional: Option<Box<str>>,
}

impl Identifier {
    #[must_use]
    pub const fn plugin(plugin: PluginName) -> Self {
        Self {
            plugin,
            data: None,
            data_additional: None,
        }
    }

    #[must_use]
    pub const fn data(plugin: PluginName, data: Box<str>) -> Self {
        Self {
            plugin,
            data: Some(data),
            data_additional: None,
        }
    }

    #[must_use]
    pub const fn data_additional(
        plugin: PluginName,
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
    // Keep the existing external IPC numeric contract separate from private UI keys.
    WorkspaceID(i32),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[cfg(test)]
mod workspace_wire_contract_tests {
    use super::WindowsOverride;

    #[test]
    fn external_workspace_override_keeps_the_existing_i32_contract() {
        let original = WindowsOverride::WorkspaceID(5);
        assert_eq!(
            serde_json::to_string(&original).expect("valid test fixture"),
            r#"{"WorkspaceID":5}"#
        );
        assert_eq!(
            serde_json::from_str::<WindowsOverride>(r#"{"WorkspaceID":5}"#)
                .expect("valid test fixture"),
            original
        );
        assert!(serde_json::from_str::<WindowsOverride>(r#"{"WorkspaceID":4294967296}"#).is_err());
    }
}
