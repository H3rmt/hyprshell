use std::path::Path;

use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub version: u64,
    #[default(None)]
    pub windows: Option<Windows>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Windows {
    #[default = 8.5]
    pub scale: f64,
    #[default = 5]
    pub items_per_row: u8,
    #[default(None)]
    pub overview: Option<Overview>,
    #[default(None)]
    pub switch: Option<Switch>,
    #[default(None)]
    pub switch_2: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Overview {
    pub launcher: Launcher,
    #[default = "Super_L"]
    pub key: Box<str>,
    #[default = 430]
    pub top_offset: u16,
    #[default(crate::Modifier::Super)]
    pub modifier: crate::Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<crate::io::FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
    #[default = "special:.*"]
    pub exclude_workspaces: Box<str>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Launcher {
    #[default(None)]
    pub default_terminal: Option<Box<str>>,
    #[default(crate::Modifier::Ctrl)]
    pub launch_modifier: crate::Modifier,
    #[default = 650]
    pub width: u16,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default(Plugins{
        applications: Some(ApplicationsPluginConfig::default()),
        terminal: Some(crate::io::EmptyConfig::default()),
        shell: None,
        websearch: Some(WebSearchConfig::default()),
        calc: Some(CalcPluginConfig::default()),
        path: Some(crate::io::EmptyConfig::default()),
        actions: Some(ActionsPluginConfig::default()),
    })]
    pub plugins: Plugins,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    pub applications: Option<ApplicationsPluginConfig>,
    pub terminal: Option<crate::io::EmptyConfig>,
    pub shell: Option<crate::io::EmptyConfig>,
    pub websearch: Option<WebSearchConfig>,
    pub calc: Option<CalcPluginConfig>,
    pub path: Option<crate::io::EmptyConfig>,
    pub actions: Option<ActionsPluginConfig>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationsPluginConfig {
    #[default = 8]
    pub run_cache_weeks: u8,
    #[default = true]
    pub show_execs: bool,
    #[default = true]
    pub show_actions_submenu: bool,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionsPluginActionCustom {
    pub names: Vec<Box<str>>,
    pub details: Box<str>,
    pub command: Box<str>,
    pub icon: Box<Path>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchEngine {
    pub url: Box<str>,
    pub name: Box<str>,
    pub key: char,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CalcPluginConfig {
    #[default(None)]
    pub prefix: Option<String>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Switch {
    #[default(crate::Modifier::Alt)]
    pub modifier: crate::Modifier,
    #[default = "Tab"]
    pub key: Box<str>,
    #[default(vec![crate::io::FilterBy::CurrentMonitor])]
    pub filter_by: Vec<crate::io::FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
    #[default = ""]
    pub exclude_workspaces: Box<str>,
    #[default = 'q']
    pub kill_key: char,
}
