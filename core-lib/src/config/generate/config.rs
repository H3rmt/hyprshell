use crate::config::generate::tui::{WEB_SEARCH_ENGINES, configurable_launcher_plugins};
use crate::config::structs::{
    Config, KeyMaybeMod, Launcher, Mod, Navigate, OpenOverview, OpenSwitch, Overview, Reverse,
    Switch, Windows,
};
use crate::config::{Plugins, WebSearchConfig};
use std::path::Path;
use tracing::warn;

pub fn get_overrides(force: &[String]) -> (bool, bool) {
    // force contains "config" or "css" or "all"
    let mut override_config = false;
    let mut override_css = false;
    for item in force {
        match item.as_str() {
            "config" => override_config = true,
            "css" => override_css = true,
            "all" => {
                override_config = true;
                override_css = true;
            }
            _ => {}
        }
    }
    (override_config, override_css)
}

pub fn check_file_exist(
    config_path: &Path,
    css_path: &Path,
    override_config: bool,
    override_css: bool,
) -> anyhow::Result<()> {
    if !override_config && config_path.exists() {
        warn!(
            "Config file {config_path:?} already exists, use -f to override all or -f config to override only the config file"
        )
    }
    if !override_css && css_path.exists() {
        warn!(
            "CSS file {css_path:?} already exists, use -f to override all or -f css to override only the css file"
        )
    }
    Ok(())
}

#[derive(Debug)]
pub struct ConfigData {
    pub enable_launcher: bool,
    pub default_terminal: Option<Box<str>>,
    pub overview: Option<(Mod, KeyMaybeMod)>,
    pub switch: Option<Mod>,
    pub launcher_plugins: Vec<Box<str>>,
    pub launcher_engines: Vec<Box<str>>,
    pub grave_reverse: bool,
}

pub fn generate_config(data: ConfigData) -> Config {
    Config {
        launcher: if data.enable_launcher {
            Some(Launcher {
                default_terminal: data.default_terminal,
                plugins: Plugins {
                    applications: data
                        .launcher_plugins
                        .iter()
                        .find(|pl| pl.as_ref().eq(configurable_launcher_plugins::APPLICATIONS))
                        .map(|_| Default::default()),
                    terminal: data
                        .launcher_plugins
                        .iter()
                        .find(|pl| pl.as_ref().eq(configurable_launcher_plugins::TERMINAL))
                        .map(|_| Default::default()),
                    shell: data
                        .launcher_plugins
                        .iter()
                        .find(|pl| pl.as_ref().eq(configurable_launcher_plugins::SHELL))
                        .map(|_| Default::default()),
                    websearch: data
                        .launcher_plugins
                        .iter()
                        .find(|pl| pl.as_ref().eq(configurable_launcher_plugins::WEB_SEARCH))
                        .map(|_| WebSearchConfig {
                            engines: data
                                .launcher_engines
                                .iter()
                                .filter_map(|engine| {
                                    WEB_SEARCH_ENGINES
                                        .iter()
                                        .find(|(name, _)| *name == engine.as_ref())
                                        .map(|(_, constructor)| constructor())
                                })
                                .collect(),
                        }),
                    calc: data
                        .launcher_plugins
                        .iter()
                        .find(|pl| pl.as_ref().eq(configurable_launcher_plugins::CALC))
                        .map(|_| Default::default()),
                },
                ..Default::default()
            })
        } else {
            None
        },
        windows: Some(Windows {
            overview: if let Some(overview) = data.overview {
                Some(Overview {
                    open: OpenOverview {
                        modifier: overview.0,
                        key: overview.1,
                    },
                    navigate: Navigate {
                        reverse: if data.grave_reverse {
                            Reverse::Key("grave".to_string())
                        } else {
                            Reverse::Mod(Mod::Shift)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
            } else {
                None
            },
            switch: if let Some(switch_mod) = data.switch {
                Some(Switch {
                    open: OpenSwitch {
                        modifier: switch_mod,
                    },
                    navigate: Navigate {
                        reverse: if data.grave_reverse {
                            Reverse::Key("grave".to_string())
                        } else {
                            Reverse::Mod(Mod::Shift)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
            } else {
                None
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}
