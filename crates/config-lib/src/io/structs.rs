use crate::{Modifier, edit_sync::sync_document};
use anyhow::Context;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use smart_default::SmartDefault;
use std::path::Path;

#[derive(Debug)]
pub struct ConfigFile {
    backing_document: toml_edit::DocumentMut,
    config: Config,
}

impl TryFrom<toml_edit::DocumentMut> for ConfigFile {
    type Error = anyhow::Error;
    fn try_from(doc: toml_edit::DocumentMut) -> Result<Self, Self::Error> {
        let config = Config::deserialize(doc.clone().into_deserializer())
            .context("Failed to deserialize Config from TOML document");
        Ok(Self {
            backing_document: doc,
            config: config?,
        })
    }
}

impl ConfigFile {
    pub fn new_without_file(config: Config) -> Self {
        Self {
            backing_document: toml_edit::DocumentMut::new(),
            config,
        }
    }

    pub fn to_string_pretty(&mut self) -> anyhow::Result<String> {
        self.sync_config_to_document()
            .context("failed to sync config in document")?;
        Ok(self.backing_document.to_string())
    }

    fn sync_config_to_document(&mut self) -> anyhow::Result<()> {
        let generated =
            toml_edit::ser::to_document(&self.config).expect("config should always serialize");

        sync_document(&mut self.backing_document, &generated);
        // TODO add sync back in for all

        // sync_array(
        //     self.backing_document
        //         .get_mut("class_to_icon")
        //         .expect("class_to_icon exists in struct"),
        //     generated
        //         .get("class_to_icon")
        //         .expect("class_to_icon exists in struct"),
        //     |i| i.get("class").map(|i| i.to_string()),
        // )
        // .context("failed to sync config into original toml config")?;
        Ok(())
    }
}

impl TryInto<crate::Config> for ConfigFile {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<crate::Config, Self::Error> {
        self.config.try_into()
    }
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[default(crate::CURRENT_CONFIG_VERSION)]
    pub version: u64,
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
    #[default = 300]
    pub live_preview_refresh_rate: u16,
    #[default(None)]
    pub overview: Option<Overview>,
    #[default(None)]
    pub switch: Option<Switch>,
    #[default(None)]
    pub switch_2: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Overview {
    pub launcher: Launcher,
    #[default = "Super_L"]
    pub key: Box<str>,
    #[default = 410]
    pub top_offset: u16,
    #[default(Modifier::Super)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = "special:.*"]
    pub exclude_workspaces: Box<str>,
    #[default = true]
    pub live_preview: bool,
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
    pub width: u16,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default(Plugins{
        applications: Some(ApplicationsPluginConfig::default()),
        terminal: Some(EmptyConfig::default()),
        shell: None,
        websearch: Some(WebSearchConfig::default()),
        calc: Some(CalcPluginConfig::default()),
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
    pub calc: Option<CalcPluginConfig>,
    pub path: Option<EmptyConfig>,
    pub actions: Option<ActionsPluginConfig>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct ApplicationsPluginConfig {
    #[default = 8]
    pub run_cache_weeks: u8,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct EmptyConfig {}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct CalcPluginConfig {
    #[default(None)]
    pub prefix: Option<String>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct ActionsPluginConfig {
    #[default(vec![
        ActionsPluginAction::Preset(ActionsPluginActionPreset::LockScreen),
        ActionsPluginAction::Preset(ActionsPluginActionPreset::Hibernate),
        ActionsPluginAction::Preset(ActionsPluginActionPreset::Logout),
        ActionsPluginAction::Preset(ActionsPluginActionPreset::Reboot),
        ActionsPluginAction::Preset(ActionsPluginActionPreset::Shutdown),
        ActionsPluginAction::Preset(ActionsPluginActionPreset::Suspend),
        ActionsPluginAction::Custom(ActionsPluginActionCustom {
            name: "Kill".into(),
            details: "Kill a process by name".into(),
            command: "pkill \"{}\" && notify-send hyprshell \"stopped {}\"".into(),
            icon: Some(Box::from(Path::new("remove"))),
        }),
        ActionsPluginAction::Custom(ActionsPluginActionCustom {
            name: "Reload Hyprshell".into(),
            details: "Reload Hyprshell".into(),
            command: "sleep 1; hyprshell socat '\"Restart\"' && notify-send \"Reloaded hyprshell\"".into(),
            icon: Some(Box::from(Path::new("system-restart"))),
        }),
    ])]
    pub actions: Vec<ActionsPluginAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ActionsPluginAction {
    Preset(ActionsPluginActionPreset),
    Custom(ActionsPluginActionCustom),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionsPluginActionPreset {
    LockScreen,
    Hibernate,
    Logout,
    Reboot,
    Shutdown,
    Suspend,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionsPluginActionCustom {
    pub name: Box<str>,
    pub details: Box<str>,
    pub command: Box<str>,
    pub icon: Option<Box<Path>>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct WebSearchConfig {
    #[default(vec![
        WebSearch::Preset(WebSearchPreset::Google),
        WebSearch::Preset(WebSearchPreset::Wikipedia)
    ])]
    pub engines: Vec<WebSearch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WebSearch {
    Preset(WebSearchPreset),
    Custom(WebSearchCustom),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchPreset {
    Google,
    Wikipedia,
    Reddit,
    Startpage,
    DuckDuckGo,
    Bing,
    YouTube,
    ChatGpt,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WebSearchCustom {
    pub url: Box<str>,
    pub name: Box<str>,
    pub key: char,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Switch {
    #[default(Modifier::Alt)]
    pub modifier: Modifier,
    #[default = "Tab"]
    pub key: Box<str>,
    #[default(vec![FilterBy::CurrentMonitor])]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub switch_workspaces: bool,
    #[default = ""]
    pub exclude_workspaces: Box<str>,
    #[default = 'q']
    pub kill_key: char,
    #[default = true]
    pub live_preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}
