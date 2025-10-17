use crate::Modifier;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[cfg_attr(
        not(feature = "no-default-config-values"),
        default(Some(crate::CURRENT_CONFIG_VERSION))
    )]
    #[cfg_attr(feature = "no-default-config-values", default(None))]
    pub version: Option<u16>,
    #[default(None)]
    pub windows: Option<Windows>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Windows {
    #[default = 8.5]
    pub scale: f64,
    #[default = 5]
    pub items_per_row: u8,
    #[default(None)]
    pub overview: Option<Overview>,
    #[default(None)]
    pub switch: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Overview {
    pub launcher: Launcher,
    #[default = "super_l"]
    pub key: Box<str>,
    #[default(Modifier::Super)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
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
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct EmptyConfig {}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
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
            icon: "remove".into(),
        }),
        ActionsPluginAction::Custom(ActionsPluginActionCustom {
            names: vec!["Reload Hyprshell".into()],
            details: "Reload Hyprshell".into(),
            command: "sleep 1; hyprshell socat '\"Restart\"'".into(),
            icon: "system-restart".into(),
        }),
    ])]
    pub actions: Vec<ActionsPluginAction>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
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
    pub icon: Box<str>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
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

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Switch {
    #[default(Modifier::Alt)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}
