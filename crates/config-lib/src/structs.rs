use crate::Modifier;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::Path;

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[default(crate::CURRENT_CONFIG_VERSION)]
    pub version: u16,
    #[default(None)]
    pub windows: Option<Windows>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Windows {
    #[default = 8.5]
    pub scale: f64,
    #[default = 5]
    pub items_per_row: u8,
    #[default(None)]
    pub overview: Option<Overview>,
    #[default(Vec::new())]
    pub switches: Vec<Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Overview {
    pub launcher: Launcher,
    #[default = "Super_L"]
    pub key: Box<str>,
    #[default(Modifier::Super)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = "special:.*"]
    pub exclude_workspaces: Box<str>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Launcher {
    #[default(None)]
    pub default_terminal: Option<Box<str>>,
    #[default(Modifier::Ctrl)]
    pub launch_modifier: Modifier,
    #[default = 650]
    pub width: u32,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default(Plugins{
        applications: Some(ApplicationsPluginConfig::default()),
        terminal: Some(EmptyConfig::default()),
        shell: None,
        websearch: Some(WebSearchConfig::default()),
        calc: Some(EmptyConfig::default()),
        path: Some(EmptyConfig::default()),
        actions: Some(ActionsPluginConfig::default()),
    })]
    pub plugins: Plugins,
}

// no default for this, if some elements are missing, they should be None.
// if no config for plugins is provided, use the default value from the launcher.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    pub applications: Option<ApplicationsPluginConfig>,
    pub terminal: Option<EmptyConfig>,
    pub shell: Option<EmptyConfig>,
    pub websearch: Option<WebSearchConfig>,
    pub calc: Option<EmptyConfig>,
    pub path: Option<EmptyConfig>,
    pub actions: Option<ActionsPluginConfig>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct EmptyConfig {}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct ActionsPluginConfig {
    #[default(vec![
        ActionsPluginAction::LockScreen,
        ActionsPluginAction::Hibernate,
        ActionsPluginAction::Logout,
        ActionsPluginAction::Reboot,
        ActionsPluginAction::Shutdown,
        ActionsPluginAction::Suspend,
        ActionsPluginAction::Custom(ActionsPluginActionCustom {
            names: vec!["Kill".into(), "Stop".into()],
            details: "Kill or stop a process by name".into(),
            command: "pkill \"{}\" && notify-send hyprshell \"stopped {}\"".into(),
            icon: Box::from(Path::new("remove")),
        }),
        ActionsPluginAction::Custom(ActionsPluginActionCustom {
            names: vec!["Reload Hyprshell".into()],
            details: "Reload Hyprshell".into(),
            command: "sleep 1; hyprshell socat '\"Restart\"'".into(),
            icon: Box::from(Path::new("system-restart")),
        }),
    ])]
    pub actions: Vec<ActionsPluginAction>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct ApplicationsPluginConfig {
    #[default = 8]
    pub run_cache_weeks: u8,
    #[default = true]
    pub show_execs: bool,
    #[default = true]
    pub show_actions_submenu: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionsPluginAction {
    LockScreen,
    Hibernate,
    Logout,
    Reboot,
    Shutdown,
    Suspend,
    Custom(ActionsPluginActionCustom),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionsPluginActionCustom {
    pub names: Vec<Box<str>>,
    pub details: Box<str>,
    pub command: Box<str>,
    pub icon: Box<Path>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct WebSearchConfig {
    #[default(vec![SearchEngine {
        url: "https://www.google.com/search?q={}".into(),
        name: "Google".into(),
        key: 'g',
    }, SearchEngine {
        url: "https://en.wikipedia.org/wiki/Special:Search?search={}".into(),
        name: "Wikipedia".into(),
        key: 'w',
    }])]
    pub engines: Vec<SearchEngine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchEngine {
    pub url: Box<str>,
    pub name: Box<str>,
    pub key: char,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KeyMod {
    Alt,
    Ctrl,
    Super,
    Shift,
}

#[allow(clippy::must_use_candidate)]
impl KeyMod {
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Alt => "alt",
            Self::Ctrl => "ctrl",
            Self::Super => "super",
            Self::Shift => "shift",
        }
    }
}

impl<'de> Deserialize<'de> for KeyMod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModVisitor;
        impl serde::de::Visitor<'_> for ModVisitor {
            type Value = KeyMod;
            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.write_str("one of: alt, ctrl, super, shift")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .try_into()
                    .map_err(|_e| E::unknown_variant(value, &["alt", "ctrl", "super", "shift"]))
            }
        }
        deserializer.deserialize_str(ModVisitor)
    }
}

impl Serialize for KeyMod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl TryFrom<&str> for KeyMod {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "alt" => Ok(Self::Alt),
            "ctrl" | "control" => Ok(Self::Ctrl),
            "super" | "win" | "windows" | "meta" => Ok(Self::Super),
            "shift" => Ok(Self::Shift),
            other => anyhow::bail!("Invalid key modifier: {other}"),
        }
    }
}

impl std::fmt::Display for KeyMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alt => write!(f, "Alt"),
            Self::Ctrl => write!(f, "Ctrl"),
            Self::Super => write!(f, "Super"),
            Self::Shift => write!(f, "Shift"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyCombo {
    pub mods: Vec<KeyMod>,
    pub key: Box<str>,
    pub hold_mods: Option<Vec<KeyMod>>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct SwitchBinds {
    #[default(vec![KeyCombo {
        mods: vec![KeyMod::Alt],
        key: Box::from("Tab"),
        hold_mods: None,
    }])]
    pub forward: Vec<KeyCombo>,
    #[default(vec![
        KeyCombo {
            mods: vec![KeyMod::Alt, KeyMod::Shift],
            key: Box::from("Tab"),
            hold_mods: None,
        },
        KeyCombo {
            mods: vec![KeyMod::Alt],
            key: Box::from("grave"),
            hold_mods: None,
        },
    ])]
    pub reverse: Vec<KeyCombo>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Switch {
    pub binds: SwitchBinds,
    #[default(vec![FilterBy::CurrentMonitor])]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
    #[default = ""]
    pub exclude_workspaces: Box<str>,
    #[default = 'q']
    pub kill_key: char,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}
