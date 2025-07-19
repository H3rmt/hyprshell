use crate::generate::tui::{WEB_SEARCH_ENGINES, configurable_launcher_plugins};
use crate::structs::{Switch, Windows};
use crate::{Config, Launcher, Modifier, Overview, Plugins, WebSearchConfig};
use std::path::Path;

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
        eprintln!(
            "\x1b[31mConfig file {config_path:?} already exists, use -f to override all or -f config to override only the config file\x1b[0m"
        )
    }
    if !override_css && css_path.exists() {
        eprintln!(
            "\x1b[31mCSS file {css_path:?} already exists, use -f to override all or -f css to override only the css file\x1b[0m"
        )
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub default_terminal: Option<Box<str>>,
    pub overview: Option<(Modifier, Box<str>)>,
    pub switch: (Option<Modifier>, bool),
    pub launcher_plugins: Vec<Box<str>>,
    pub launcher_engines: Vec<Box<str>>,
}

pub fn generate_config(data: ConfigData) -> Config {
    Config {
        windows: Some(Windows {
            overview: if let Some(overview) = data.overview {
                Some(Overview {
                    modifier: overview.0,
                    key: overview.1,
                    launcher: Launcher {
                        default_terminal: data.default_terminal,
                        plugins: Plugins {
                            applications: data
                                .launcher_plugins
                                .iter()
                                .find(|pl| {
                                    pl.as_ref().eq(configurable_launcher_plugins::APPLICATIONS)
                                })
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
                                .find(|pl| {
                                    pl.as_ref().eq(configurable_launcher_plugins::WEB_SEARCH)
                                })
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
                            path: Some(Default::default()),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
            } else {
                None
            },
            switch: data.switch.0.map(|switch_mod| Switch {
                modifier: switch_mod,
                show_workspaces: data.switch.1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::tui::configurable_launcher_plugins;

    fn assert_config_matches_data(config: &Config, data: &ConfigData) {
        if let Some(windows) = &config.windows {
            if let Some(overview) = &windows.overview {
                assert_eq!(overview.modifier, data.overview.as_ref().unwrap().0);
                assert_eq!(overview.key, data.overview.as_ref().unwrap().1);
                assert_eq!(overview.launcher.default_terminal, data.default_terminal);

                let plugins = &overview.launcher.plugins;
                assert_eq!(
                    plugins.applications.is_some(),
                    data.launcher_plugins
                        .iter()
                        .any(|p| p.as_ref() == configurable_launcher_plugins::APPLICATIONS)
                );
                assert_eq!(
                    plugins.terminal.is_some(),
                    data.launcher_plugins
                        .iter()
                        .any(|p| p.as_ref() == configurable_launcher_plugins::TERMINAL)
                );
                assert_eq!(
                    plugins.shell.is_some(),
                    data.launcher_plugins
                        .iter()
                        .any(|p| p.as_ref() == configurable_launcher_plugins::SHELL)
                );
                assert_eq!(
                    plugins.websearch.is_some(),
                    data.launcher_plugins
                        .iter()
                        .any(|p| p.as_ref() == configurable_launcher_plugins::WEB_SEARCH)
                );
                assert_eq!(
                    plugins.calc.is_some(),
                    data.launcher_plugins
                        .iter()
                        .any(|p| p.as_ref() == configurable_launcher_plugins::CALC)
                );
            }
            if let Some(switch) = &windows.switch {
                assert_eq!(switch.modifier, data.switch.0.unwrap());
                assert_eq!(switch.show_workspaces, data.switch.1);
            }
        }
    }

    #[test]
    fn test_empty_config() {
        let data = ConfigData {
            default_terminal: None,
            overview: None,
            switch: (None, false),
            launcher_plugins: vec![],
            launcher_engines: vec![],
        };

        let config = generate_config(data);
        assert!(config.windows.as_ref().unwrap().overview.is_none());
        assert!(config.windows.unwrap().switch.is_none());
    }

    #[test]
    fn test_full_config() {
        let data = ConfigData {
            default_terminal: Some("alacritty".into()),
            overview: Some((Modifier::Super, "super_l".into())),
            switch: (Some(Modifier::Alt), true),
            launcher_plugins: vec![
                configurable_launcher_plugins::APPLICATIONS.into(),
                configurable_launcher_plugins::TERMINAL.into(),
                configurable_launcher_plugins::SHELL.into(),
                configurable_launcher_plugins::WEB_SEARCH.into(),
                configurable_launcher_plugins::CALC.into(),
            ],
            launcher_engines: vec!["Google".into(), "Wikipedia".into()],
        };

        let config = generate_config(data.clone());
        assert_config_matches_data(&config, &data);
    }

    #[test]
    fn test_partial_config() {
        let data = ConfigData {
            default_terminal: Some("xterm".into()),
            overview: Some((Modifier::Ctrl, "ctrl_l".into())),
            switch: (Some(Modifier::Shift), false),
            launcher_plugins: vec![
                configurable_launcher_plugins::APPLICATIONS.into(),
                configurable_launcher_plugins::CALC.into(),
            ],
            launcher_engines: vec![],
        };

        let config = generate_config(data.clone());
        assert_config_matches_data(&config, &data);
    }
}
