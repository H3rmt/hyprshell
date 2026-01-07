use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[default(crate::migrate::m3t4::PREV_CONFIG_VERSION)]
    pub version: u16,
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
    pub launcher: crate::Launcher,
    #[default = "Super_L"]
    pub key: Box<str>,
    #[default(crate::Modifier::Super)]
    pub modifier: crate::Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
    #[default = ""]
    pub exclude_special_workspaces: Box<str>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Switch {
    #[default(crate::Modifier::Alt)]
    pub modifier: crate::Modifier,
    #[default = "Tab"]
    pub key: Box<str>,
    #[default(vec![crate::FilterBy::CurrentMonitor])]
    pub filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
    #[default = ""]
    pub exclude_special_workspaces: Box<str>,
    #[default = 'q']
    pub kill_key: char,
}
