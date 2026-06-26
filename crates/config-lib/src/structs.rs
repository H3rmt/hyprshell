use crate::Modifier;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub windows: Option<Windows>,
}

impl Default for Config {
    fn default() -> Self {
        crate::io::Config::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Windows {
    pub general: WindowsGeneral,
    pub overview: Option<Overview>,
    pub switch: Option<Switch>,
    pub switch_2: Option<Switch>,
}

impl Default for Windows {
    fn default() -> Self {
        crate::io::Windows::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowsGeneral {
    pub scale: f64,
    pub items_per_row: u8,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Overview {
    pub launcher: Launcher,
    pub key: Box<str>,
    pub top_offset: u16,
    pub modifier: Modifier,
    pub filter_by_same_class: bool,
    pub filter_by_current_workspace: bool,
    pub filter_by_current_monitor: bool,
    pub exclude_workspaces: Box<str>,
}

impl Default for Overview {
    fn default() -> Self {
        crate::io::Overview::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Launcher {
    pub default_terminal: Option<Box<str>>,
    pub launch_modifier: Modifier,
    pub width: u16,
    pub max_items: u8,
    pub show_when_empty: bool,
    pub plugins: Plugins,
}

impl Default for Launcher {
    fn default() -> Self {
        crate::io::Launcher::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plugins {
    pub applications: Option<ApplicationsPluginConfig>,
    pub terminal: Option<()>,
    pub shell: Option<()>,
    pub websearch: Option<WebSearchConfig>,
    pub calc: Option<CalcPluginConfig>,
    pub path: Option<()>,
    pub actions: Option<ActionsPluginConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionsPluginConfig {
    pub actions: Vec<ActionsPluginAction>,
}

impl Default for ActionsPluginConfig {
    fn default() -> Self {
        crate::io::ActionsPluginConfig::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ApplicationsPluginConfig {
    pub run_cache_weeks: u8,
    pub show_execs: bool,
    pub show_actions_submenu: bool,
}

impl Default for ApplicationsPluginConfig {
    fn default() -> Self {
        crate::io::ApplicationsPluginConfig::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActionsPluginAction {
    LockScreen,
    Logout,
    Hibernate,
    Reboot,
    Shutdown,
    Suspend,
    Custom(ActionsPluginActionCustom),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionsPluginActionCustom {
    pub names: Vec<Box<str>>,
    pub details: Box<str>,
    pub command: Box<str>,
    pub icon: Box<Path>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebSearchConfig {
    pub engines: Vec<SearchEngine>,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        crate::io::WebSearchConfig::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SearchEngine {
    pub url: Box<str>,
    pub name: Box<str>,
    pub key: char,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CalcPluginConfig {
    pub prefix: Option<String>,
}

impl Default for CalcPluginConfig {
    fn default() -> Self {
        crate::io::CalcPluginConfig::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Switch {
    pub modifier: Modifier,
    pub key: Box<str>,
    pub filter_by_same_class: bool,
    pub filter_by_current_workspace: bool,
    pub filter_by_current_monitor: bool,
    pub switch_workspaces: bool,
    pub exclude_workspaces: Box<str>,
    pub kill_key: char,
}

impl Default for Switch {
    fn default() -> Self {
        crate::io::Switch::default()
            .try_into()
            .expect("the default config invalid")
    }
}
