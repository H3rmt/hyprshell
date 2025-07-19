use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub(super) layerrules: bool,
    pub(super) kill_bind: String,
    pub(super) windows: Option<Windows>,
    #[allow(dead_code)]
    pub(super) version: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Windows {
    pub(super) scale: f64,
    pub(super) items_per_row: u8,
    pub(super) overview: Option<Overview>,
    pub(super) switch: Option<Switch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Switch {
    pub modifier: Modifier,
    pub filter_by: Vec<crate::FilterBy>,
    pub show_workspaces: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Overview {
    pub(super) launcher: Launcher,
    pub(super) key: Box<str>,
    pub(super) modifier: Modifier,
    pub(super) filter_by: Vec<crate::FilterBy>,
    pub(super) hide_filtered: bool,
    #[allow(dead_code)]
    pub(super) strip_html_from_workspace_title: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Launcher {
    pub(super) default_terminal: Option<Box<str>>,
    pub(super) launch_modifier: Modifier,
    pub(super) width: u32,
    pub(super) max_items: u8,
    pub(super) show_when_empty: bool,
    #[allow(dead_code)]
    pub(super) animate_launch_ms: u64,
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
            Modifier::Alt => write!(f, "alt"),
            Modifier::Ctrl => write!(f, "ctrl"),
            Modifier::Super => write!(f, "super"),
            Modifier::Shift => write!(f, "shift"),
        }
    }
}
