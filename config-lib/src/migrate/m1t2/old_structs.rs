use serde::Deserialize;
use smart_default::SmartDefault;
use std::fmt::Display;

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[default = true]
    pub(super) layerrules: bool,
    #[default = "ctrl+shift+alt, h"]
    pub(super) kill_bind: String,
    #[default(None)]
    pub(super) windows: Option<Windows>,
    #[allow(dead_code)]
    pub(super) version: u16,
}

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Windows {
    #[default = 8.5]
    pub(super) scale: f64,
    #[default = 5]
    pub(super) items_per_row: u8,
    #[default(None)]
    pub(super) overview: Option<Overview>,
    #[default(None)]
    pub(super) switch: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Switch {
    #[default(Modifier::Alt)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub show_workspaces: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Overview {
    pub(super) launcher: Launcher,
    #[default = "super_l"]
    pub(super) key: Box<str>,
    #[default(Modifier::Super)]
    pub(super) modifier: Modifier,
    #[default(Vec::new())]
    pub(super) filter_by: Vec<crate::FilterBy>,
    #[default = false]
    pub(super) hide_filtered: bool,
    #[allow(dead_code)]
    #[default = true]
    pub(super) strip_html_from_workspace_title: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Launcher {
    #[default(None)]
    pub(super) default_terminal: Option<Box<str>>,
    #[default(Modifier::Ctrl)]
    pub(super) launch_modifier: Modifier,
    #[default = 650]
    pub(super) width: u32,
    #[default = 5]
    pub(super) max_items: u8,
    #[default = true]
    pub(super) show_when_empty: bool,
    #[allow(dead_code)]
    #[default = 400]
    pub(super) animate_launch_ms: u64,
    #[default(crate::Plugins{
        applications: Some(crate::ApplicationsPluginConfig::default()),
        terminal: Some(crate::EmptyConfig::default()),
        shell: None,
        websearch: Some(crate::WebSearchConfig::default()),
        calc: Some(crate::EmptyConfig::default()),
        path: Some(crate::EmptyConfig::default()),
    })]
    pub(super) plugins: crate::Plugins,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Modifier {
    Alt,
    Ctrl,
    Super,
    Shift,
}

impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alt => write!(f, "alt"),
            Self::Ctrl => write!(f, "ctrl"),
            Self::Super => write!(f, "super"),
            Self::Shift => write!(f, "shift"),
        }
    }
}
