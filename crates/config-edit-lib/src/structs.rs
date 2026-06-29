use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct Config {
    pub windows: Windows,
}

#[derive(Debug, Clone)]
pub struct Windows {
    pub enabled: bool,
    pub scale: f64,
    pub items_per_row: u8,
    pub overview: Overview,
    pub switch: Switch,
    pub switch_2: Switch,
}

#[derive(Debug, Clone)]
pub struct Overview {
    pub enabled: bool,
    pub launcher: Launcher,
    pub key: String,
    pub top_offset: u16,
    pub modifier: ConfigModifier,
    pub same_class: bool,
    pub current_workspace: bool,
    pub current_monitor: bool,
    pub exclude_workspaces: String,
}

#[derive(Debug, Clone)]
pub struct Launcher {
    pub default_terminal: Option<String>,
    pub launch_modifier: ConfigModifier,
    pub width: u16,
    pub max_items: u8,
    pub show_when_empty: bool,
    pub plugins: Plugins,
}

#[derive(Debug, Clone)]
pub struct Plugins {
    pub applications: ApplicationsPluginConfig,
    pub terminal: EmptyConfig,
    pub shell: EmptyConfig,
    pub websearch: WebSearchConfig,
    pub calc: CalcPluginConfig,
    pub path: EmptyConfig,
    pub actions: ActionsPluginConfig,
}

#[derive(Debug, Clone)]
pub struct WebSearchConfig {
    pub enabled: bool,
    pub engines: Vec<config_lib::SearchEngine>,
}

#[derive(Debug, Clone)]
pub struct EmptyConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CalcPluginConfig {
    pub enabled: bool,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActionsPluginConfig {
    pub enabled: bool,
    pub actions: Vec<config_lib::ActionsPluginAction>,
}

#[derive(Debug, Clone)]
pub struct ApplicationsPluginConfig {
    pub enabled: bool,
    pub run_cache_weeks: u8,
    pub show_execs: bool,
    pub show_actions_submenu: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Switch {
    pub enabled: bool,
    pub modifier: ConfigModifier,
    pub key: String,
    pub same_class: bool,
    pub current_workspace: bool,
    pub current_monitor: bool,
    pub switch_workspaces: bool,
    pub exclude_workspaces: String,
    pub kill_key: char,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConfigModifier {
    None,
    Alt,
    Super,
    Ctrl,
}

#[allow(dead_code)]
impl ConfigModifier {
    pub const fn strings() -> &'static [&'static str] {
        &["Alt", "Super", "Ctrl"]
    }
    pub const fn strings_with_none() -> &'static [&'static str] {
        &["None", "Alt", "Super", "Ctrl"]
    }
}

impl fmt::Display for ConfigModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Alt => write!(f, "Alt"),
            Self::Super => write!(f, "Super"),
            Self::Ctrl => write!(f, "Ctrl"),
        }
    }
}

impl TryFrom<u32> for ConfigModifier {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Super),
            2 => Ok(Self::Ctrl),
            3 => Ok(Self::Alt),
            _ => Err(()),
        }
    }
}

impl From<ConfigModifier> for u32 {
    fn from(value: ConfigModifier) -> Self {
        match value {
            ConfigModifier::None => 0,
            ConfigModifier::Super => 1,
            ConfigModifier::Ctrl => 2,
            ConfigModifier::Alt => 3,
        }
    }
}

impl From<config_lib::Modifier> for ConfigModifier {
    fn from(value: config_lib::Modifier) -> Self {
        match value {
            config_lib::Modifier::None => Self::None,
            config_lib::Modifier::Alt => Self::Alt,
            config_lib::Modifier::Super => Self::Super,
            config_lib::Modifier::Ctrl => Self::Ctrl,
        }
    }
}

impl From<ConfigModifier> for config_lib::Modifier {
    fn from(value: ConfigModifier) -> Self {
        match value {
            ConfigModifier::None => Self::None,
            ConfigModifier::Alt => Self::Alt,
            ConfigModifier::Super => Self::Super,
            ConfigModifier::Ctrl => Self::Ctrl,
        }
    }
}

impl From<config_lib::Config> for Config {
    fn from(value: config_lib::Config) -> Self {
        Self {
            windows: value.windows.into(),
        }
    }
}

impl From<Config> for config_lib::Config {
    fn from(value: Config) -> Self {
        Self {
            windows: value.windows.into(),
        }
    }
}

impl From<Option<config_lib::Windows>> for Windows {
    fn from(value: Option<config_lib::Windows>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            scale: v.general.scale,
            items_per_row: v.general.items_per_row,
            overview: v.overview.into(),
            switch: v.switch.into(),
            switch_2: v.switch_2.into(),
        }
    }
}

impl From<Windows> for Option<config_lib::Windows> {
    fn from(value: Windows) -> Self {
        if value.enabled {
            Some(config_lib::Windows {
                general: config_lib::WindowsGeneral {
                    scale: value.scale,
                    items_per_row: value.items_per_row,
                },
                overview: value.overview.into(),
                switch: value.switch.into(),
                switch_2: value.switch_2.into(),
            })
        } else {
            None
        }
    }
}

impl From<Option<config_lib::Switch>> for Switch {
    fn from(value: Option<config_lib::Switch>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            modifier: v.modifier.into(),
            key: v.key.to_string(),
            same_class: v.filter_by_same_class,
            current_workspace: v.filter_by_current_workspace,
            current_monitor: v.filter_by_current_monitor,
            switch_workspaces: v.switch_workspaces,
            exclude_workspaces: v.exclude_workspaces.to_string(),
            kill_key: v.kill_key,
        }
    }
}

impl From<Switch> for Option<config_lib::Switch> {
    fn from(value: Switch) -> Self {
        if value.enabled {
            Some(config_lib::Switch {
                modifier: value.modifier.into(),
                key: Box::from(value.key),
                filter_by_same_class: value.same_class,
                filter_by_current_workspace: value.current_workspace,
                filter_by_current_monitor: value.current_monitor,
                switch_workspaces: value.switch_workspaces,
                exclude_workspaces: Box::from(value.exclude_workspaces),
                kill_key: value.kill_key,
            })
        } else {
            None
        }
    }
}

impl From<Option<config_lib::Overview>> for Overview {
    fn from(value: Option<config_lib::Overview>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            launcher: v.launcher.into(),
            key: v.key.to_string(),
            top_offset: v.top_offset,
            modifier: v.modifier.into(),
            same_class: v.filter_by_same_class,
            current_workspace: v.filter_by_current_workspace,
            current_monitor: v.filter_by_current_monitor,
            exclude_workspaces: v.exclude_workspaces.to_string(),
        }
    }
}

impl From<Overview> for Option<config_lib::Overview> {
    fn from(value: Overview) -> Self {
        if value.enabled {
            Some(config_lib::Overview {
                launcher: value.launcher.into(),
                key: Box::from(value.key),
                top_offset: value.top_offset,
                modifier: value.modifier.into(),
                filter_by_same_class: value.same_class,
                filter_by_current_workspace: value.current_workspace,
                filter_by_current_monitor: value.current_monitor,
                exclude_workspaces: Box::from(value.exclude_workspaces),
            })
        } else {
            None
        }
    }
}

impl From<config_lib::Launcher> for Launcher {
    fn from(value: config_lib::Launcher) -> Self {
        Self {
            default_terminal: value.default_terminal.map(|s| s.to_string()),
            launch_modifier: value.launch_modifier.into(),
            width: value.width,
            max_items: value.max_items,
            show_when_empty: value.show_when_empty,
            plugins: value.plugins.into(),
        }
    }
}

impl From<Launcher> for config_lib::Launcher {
    fn from(value: Launcher) -> Self {
        Self {
            default_terminal: value.default_terminal.map(Box::from),
            launch_modifier: value.launch_modifier.into(),
            alt_launch_modifier: match value.launch_modifier {
                ConfigModifier::Alt => config_lib::Modifier::Ctrl,
                _ => config_lib::Modifier::Alt,
            },
            width: value.width,
            max_items: value.max_items,
            show_when_empty: value.show_when_empty,
            plugins: value.plugins.into(),
        }
    }
}

impl From<Option<()>> for EmptyConfig {
    fn from(value: Option<()>) -> Self {
        let enabled = value.is_some();
        Self { enabled }
    }
}

impl From<EmptyConfig> for Option<()> {
    fn from(value: EmptyConfig) -> Self {
        if value.enabled { Some(()) } else { None }
    }
}

impl From<Option<config_lib::ActionsPluginConfig>> for ActionsPluginConfig {
    fn from(value: Option<config_lib::ActionsPluginConfig>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            actions: v.actions,
        }
    }
}

impl From<ActionsPluginConfig> for Option<config_lib::ActionsPluginConfig> {
    fn from(value: ActionsPluginConfig) -> Self {
        if value.enabled {
            Some(config_lib::ActionsPluginConfig {
                actions: value.actions,
            })
        } else {
            None
        }
    }
}

impl From<Option<config_lib::ApplicationsPluginConfig>> for ApplicationsPluginConfig {
    fn from(value: Option<config_lib::ApplicationsPluginConfig>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            run_cache_weeks: v.run_cache_weeks,
            show_execs: v.show_execs,
            show_actions_submenu: v.show_actions_submenu,
        }
    }
}

impl From<ApplicationsPluginConfig> for Option<config_lib::ApplicationsPluginConfig> {
    fn from(value: ApplicationsPluginConfig) -> Self {
        if value.enabled {
            Some(config_lib::ApplicationsPluginConfig {
                run_cache_weeks: value.run_cache_weeks,
                show_execs: value.show_execs,
                show_actions_submenu: value.show_actions_submenu,
            })
        } else {
            None
        }
    }
}

impl From<Option<config_lib::WebSearchConfig>> for WebSearchConfig {
    fn from(value: Option<config_lib::WebSearchConfig>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            engines: v.engines,
        }
    }
}

impl From<WebSearchConfig> for Option<config_lib::WebSearchConfig> {
    fn from(value: WebSearchConfig) -> Self {
        if value.enabled {
            Some(config_lib::WebSearchConfig {
                engines: value.engines,
            })
        } else {
            None
        }
    }
}

impl From<Option<config_lib::CalcPluginConfig>> for CalcPluginConfig {
    fn from(value: Option<config_lib::CalcPluginConfig>) -> Self {
        let enabled = value.is_some();
        let v = value.unwrap_or_default();
        Self {
            enabled,
            prefix: v.prefix,
        }
    }
}

impl From<CalcPluginConfig> for Option<config_lib::CalcPluginConfig> {
    fn from(value: CalcPluginConfig) -> Self {
        if value.enabled {
            Some(config_lib::CalcPluginConfig {
                prefix: value.prefix,
            })
        } else {
            None
        }
    }
}

impl From<config_lib::Plugins> for Plugins {
    fn from(value: config_lib::Plugins) -> Self {
        Self {
            applications: value.applications.into(),
            terminal: value.terminal.into(),
            shell: value.shell.into(),
            websearch: value.websearch.into(),
            calc: value.calc.into(),
            path: value.path.into(),
            actions: value.actions.into(),
        }
    }
}

impl From<Plugins> for config_lib::Plugins {
    fn from(value: Plugins) -> Self {
        Self {
            applications: value.applications.into(),
            terminal: value.terminal.into(),
            shell: value.shell.into(),
            websearch: value.websearch.into(),
            calc: value.calc.into(),
            path: value.path.into(),
            actions: value.actions.into(),
        }
    }
}
