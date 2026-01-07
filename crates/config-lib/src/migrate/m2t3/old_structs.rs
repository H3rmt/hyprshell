use crate::migrate::m3t4;
use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[default = true]
    pub layerrules: bool,
    #[default = "ctrl+shift+alt, h"]
    pub kill_bind: String,
    #[default(None)]
    pub windows: Option<Windows>,
    #[allow(dead_code)]
    pub version: Option<u16>,
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
    pub switch: Option<m3t4::Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Overview {
    pub launcher: Launcher,
    #[default = "super_l"]
    pub key: Box<str>,
    #[default(crate::Modifier::Super)]
    pub modifier: crate::Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Launcher {
    #[default(None)]
    pub default_terminal: Option<Box<str>>,
    #[default(crate::Modifier::Ctrl)]
    pub launch_modifier: crate::Modifier,
    #[default = 650]
    pub width: u32,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default(Plugins{
        applications: Some(crate::ApplicationsPluginConfig::default()),
        terminal: Some(crate::EmptyConfig::default()),
        shell: None,
        websearch: Some(crate::WebSearchConfig::default()),
        calc: Some(crate::EmptyConfig::default()),
        path: Some(crate::EmptyConfig::default()),
    })]
    pub plugins: Plugins,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    pub applications: Option<crate::ApplicationsPluginConfig>,
    pub terminal: Option<crate::EmptyConfig>,
    pub shell: Option<crate::EmptyConfig>,
    pub websearch: Option<crate::WebSearchConfig>,
    pub calc: Option<crate::EmptyConfig>,
    pub path: Option<crate::EmptyConfig>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[allow(dead_code)]
pub struct Switch {
    #[default(crate::Modifier::Alt)]
    pub modifier: crate::Modifier,
    #[default = "Tab"]
    pub key: Box<str>,
    #[default(vec![crate::FilterBy::CurrentMonitor])]
    pub filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
}
