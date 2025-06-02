use crate::config;
use crate::config::migrate::old_structs;

impl From<old_structs::Config> for config::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            layerrules: value.layerrules,
            launcher: value.launcher.map(Into::into),
            windows: value.windows.map(Into::into),
            ..Default::default()
        }
    }
}

impl From<old_structs::Windows> for config::Windows {
    fn from(value: old_structs::Windows) -> Self {
        // actual migration, ignoring size_factor
        Self {
            scale: value.scale,
            workspaces_per_row: value.workspaces_per_row,
            strip_html_from_workspace_title: value.strip_html_from_workspace_title,
            overview: value.overview.map(Into::into),
            switch: value.switch.map(Into::into),
        }
    }
}

impl From<old_structs::Launcher> for config::Launcher {
    fn from(value: old_structs::Launcher) -> Self {
        Self {
            max_items: value.max_items,
            width: value.width,
            animate_launch_ms: value.animate_launch_ms,
            default_terminal: value.default_terminal,
            plugins: config::Plugins {
                applications: value.plugins.iter().find_map(|p| match p {
                    old_structs::Plugin::Applications(options) => Some(options.clone()),
                    _ => None,
                }),
                terminal: value.plugins.iter().find_map(|p| match p {
                    old_structs::Plugin::Terminal() => Some(Default::default()),
                    _ => None,
                }),
                shell: value.plugins.iter().find_map(|p| match p {
                    old_structs::Plugin::Shell() => Some(Default::default()),
                    _ => None,
                }),
                websearch: value.plugins.iter().find_map(|p| match p {
                    old_structs::Plugin::WebSearch(engines) => Some(config::WebSearchConfig {
                        engines: engines.iter().map(|e| e.clone().into()).collect::<Vec<_>>(),
                    }),
                    _ => None,
                }),
                calc: value.plugins.iter().find_map(|p| match p {
                    old_structs::Plugin::Calc() => Some(Default::default()),
                    _ => None,
                }),
            },
            ..Default::default()
        }
    }
}

impl From<old_structs::SearchEngine> for config::SearchEngine {
    fn from(value: old_structs::SearchEngine) -> Self {
        Self {
            name: value.name,
            url: value.url,
            // actual migration
            key: value.key.chars().next().unwrap_or('?'),
        }
    }
}

impl From<old_structs::Overview> for config::Overview {
    fn from(value: old_structs::Overview) -> Self {
        Self {
            open: value.open,
            other: value.other,
            navigate: value.navigate.into(),
        }
    }
}
impl From<old_structs::Switch> for config::Switch {
    fn from(value: old_structs::Switch) -> Self {
        Self {
            open: value.open,
            other: value.other,
            navigate: value.navigate.into(),
        }
    }
}
impl From<old_structs::Navigate> for config::Navigate {
    fn from(value: old_structs::Navigate) -> Self {
        Self {
            forward: value.forward,
            reverse: value.reverse.into(),
        }
    }
}
impl From<old_structs::Reverse> for config::Reverse {
    fn from(value: old_structs::Reverse) -> Self {
        match value {
            old_structs::Reverse::Key(key) => config::Reverse::Key(key),
            old_structs::Reverse::Mod(modifier) => config::Reverse::Mod(modifier),
        }
    }
}
