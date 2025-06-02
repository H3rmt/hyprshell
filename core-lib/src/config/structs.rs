use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fmt::Display;

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[default = true]
    pub layerrules: bool,
    #[default = "ctrl+shift+alt, h"]
    pub kill_bind: String,
    #[default(None)]
    pub launcher: Option<Launcher>,
    #[default(Some(Windows::default()))]
    pub windows: Option<Windows>,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Launcher {
    #[default(None)]
    pub default_terminal: Option<Box<str>>,
    #[default = 650]
    pub width: u32,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default = 250]
    pub animate_launch_ms: u64,
    #[default(Plugins::default())]
    pub plugins: Plugins,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Plugins {
    #[default(Some(Default::default()))]
    pub applications: Option<ApplicationsPluginConfig>,
    #[default(Some(Default::default()))]
    pub terminal: Option<EmptyConfig>,
    #[default(None)]
    pub shell: Option<EmptyConfig>,
    #[default(Some(Default::default()))]
    pub websearch: Option<WebSearchConfig>,
    #[default(Some(Default::default()))]
    pub calc: Option<EmptyConfig>,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EmptyConfig {}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationsPluginConfig {
    #[default = 4]
    pub run_cache_weeks: u8,
    #[default = true]
    pub show_execs: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WebSearchConfig {
    #[default(vec![SearchEngine {
        url: "https://www.google.com/search?q={}".into(),
        name: "Google".into(),
        key: 'g',
    }])]
    pub engines: Vec<SearchEngine>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchEngine {
    pub url: String,
    pub name: String,
    pub key: char,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Windows {
    #[default = 8.5]
    pub scale: f64,
    #[default = 5]
    pub workspaces_per_row: u8,
    #[default = true]
    pub strip_html_from_workspace_title: bool,
    pub overview: Option<Overview>,
    pub switch: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Overview {
    pub open: OpenOverview,
    pub navigate: Navigate,
    pub other: OtherOverview,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtherOverview {
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpenOverview {
    #[default = "super"]
    pub key: KeyMaybeMod,
    #[default(Mod::Super)]
    pub modifier: Mod,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Navigate {
    #[default = "tab"]
    pub forward: String,
    #[default(Reverse::Mod(Mod::Shift))]
    pub reverse: Reverse,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Switch {
    pub open: OpenSwitch,
    pub navigate: Navigate,
    pub other: OtherSwitch,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpenSwitch {
    #[default(Mod::Super)]
    pub modifier: Mod,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtherSwitch {
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = true]
    pub hide_filtered: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reverse {
    Key(String),
    Mod(Mod),
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mod {
    Alt,
    Ctrl,
    Super,
    Shift,
}

impl Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mod::Alt => write!(f, "alt"),
            Mod::Ctrl => write!(f, "ctrl"),
            Mod::Super => write!(f, "super"),
            Mod::Shift => write!(f, "shift"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMaybeMod(String);
impl From<&str> for KeyMaybeMod {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// https://wiki.hyprland.org/Configuring/Variables/#variable-types
// SHIFT CAPS CTRL/CONTROL ALT MOD2 MOD3 SUPER/WIN/LOGO/MOD4 MOD5
impl KeyMaybeMod {
    pub fn to_key(&self) -> String {
        match &*self.0.to_ascii_lowercase() {
            "alt" => "alt_l".to_string(),
            "ctrl" => "ctrl_l".to_string(),
            "super" => "super_l".to_string(),
            "shift" => "shift_l".to_string(),
            a => a.to_string(),
        }
    }
}

impl Mod {
    pub fn mod_to_keys(&self) -> [&'static str; 2] {
        match self {
            Mod::Alt => ["alt_l", "alt_r"],
            Mod::Ctrl => ["control_l", "control_r"], // WHY is this not ctrl_l and ctrl_r?
            Mod::Super => ["super_l", "super_r"],
            Mod::Shift => ["shift_l", "shift_r"],
        }
    }
}
