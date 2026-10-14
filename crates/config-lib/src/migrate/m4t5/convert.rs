use crate::migrate::m4t5::{NEXT_CONFIG_VERSION, old_structs};

impl From<old_structs::Config> for crate::io::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            windows: value.windows.map(old_structs::Windows::into),
            version: NEXT_CONFIG_VERSION,
        }
    }
}

impl From<old_structs::Windows> for crate::io::Windows {
    fn from(value: old_structs::Windows) -> Self {
        Self {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switch: value.switch.map(old_structs::Switch::into),
            switch_2: value.switch_2.map(old_structs::Switch::into),
            overview: value.overview.map(old_structs::Overview::into),
            ..Default::default()
        }
    }
}

impl From<old_structs::Overview> for crate::io::Overview {
    fn from(value: old_structs::Overview) -> Self {
        Self {
            key: value.key,
            top_offset: value.top_offset,
            modifier: value.modifier,
            filter_by: value.filter_by,
            launcher: value.launcher.into(),
            exclude_workspaces: value.exclude_workspaces,
            ..Default::default()
        }
    }
}

impl From<old_structs::Launcher> for crate::io::Launcher {
    fn from(value: old_structs::Launcher) -> Self {
        Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier,
            max_items: value.max_items,
            width: value.width,
            show_when_empty: value.show_when_empty,
            plugins: value.plugins.into(),
        }
    }
}

impl From<old_structs::Plugins> for crate::io::Plugins {
    fn from(value: old_structs::Plugins) -> Self {
        Self {
            applications: value
                .applications
                .map(old_structs::ApplicationsPluginConfig::into),
            calc: value.calc.map(crate::io::CalcPluginConfig::from),
            terminal: value.terminal,
            shell: value.shell,
            path: value.path,
            actions: value.actions.map(old_structs::ActionsPluginConfig::into),
            websearch: value.websearch.map(old_structs::WebSearchConfig::into),
        }
    }
}

impl From<old_structs::CalcPluginConfig> for crate::io::CalcPluginConfig {
    fn from(value: old_structs::CalcPluginConfig) -> Self {
        Self {
            prefix: value.prefix,
        }
    }
}

impl From<old_structs::ApplicationsPluginConfig> for crate::io::ApplicationsPluginConfig {
    fn from(value: old_structs::ApplicationsPluginConfig) -> Self {
        Self {
            run_cache_weeks: value.run_cache_weeks,
        }
    }
}

impl From<old_structs::ActionsPluginConfig> for crate::io::ActionsPluginConfig {
    fn from(value: old_structs::ActionsPluginConfig) -> Self {
        Self {
            actions: value
                .actions
                .into_iter()
                .map(crate::io::ActionsPluginAction::from)
                .collect(),
        }
    }
}

impl From<old_structs::ActionsPluginAction> for crate::io::ActionsPluginAction {
    fn from(value: old_structs::ActionsPluginAction) -> Self {
        match value {
            super::ActionsPluginAction::Suspend => crate::io::ActionsPluginAction::Preset(
                crate::io::ActionsPluginActionPreset::Suspend,
            ),
            super::ActionsPluginAction::Hibernate => crate::io::ActionsPluginAction::Preset(
                crate::io::ActionsPluginActionPreset::Hibernate,
            ),
            super::ActionsPluginAction::LockScreen => crate::io::ActionsPluginAction::Preset(
                crate::io::ActionsPluginActionPreset::LockScreen,
            ),
            super::ActionsPluginAction::Logout => {
                crate::io::ActionsPluginAction::Preset(crate::io::ActionsPluginActionPreset::Logout)
            }
            super::ActionsPluginAction::Reboot => {
                crate::io::ActionsPluginAction::Preset(crate::io::ActionsPluginActionPreset::Reboot)
            }
            super::ActionsPluginAction::Shutdown => crate::io::ActionsPluginAction::Preset(
                crate::io::ActionsPluginActionPreset::Shutdown,
            ),
            super::ActionsPluginAction::Custom(cfg) => {
                crate::io::ActionsPluginAction::Custom(crate::io::ActionsPluginActionCustom {
                    command: cfg.command,
                    details: cfg.details,
                    icon: Some(cfg.icon),
                    name: cfg
                        .names
                        .first()
                        .cloned()
                        .unwrap_or_else(|| Box::from("unnamed")),
                })
            }
        }
    }
}

impl From<old_structs::WebSearchConfig> for crate::io::WebSearchConfig {
    fn from(value: old_structs::WebSearchConfig) -> Self {
        Self {
            engines: value
                .engines
                .into_iter()
                .map(crate::io::WebSearch::from)
                .collect(),
        }
    }
}

impl From<old_structs::SearchEngine> for crate::io::WebSearch {
    fn from(value: old_structs::SearchEngine) -> Self {
        match (&value.key, &*value.name, &*value.url) {
            ('g', "Google", "https://www.google.com/search?q={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::Google)
            }
            ('w', "Wikipedia", "https://en.wikipedia.org/wiki/Special:Search?search={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::Wikipedia)
            }
            ('r', "Reddit", "https://www.reddit.com/search?q={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::Reddit)
            }
            ('s', "Startpage", "https://www.startpage.com/sp/search?query={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::Startpage)
            }
            ('d', "DuckDuckGo", "https://duckduckgo.com/?q={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::DuckDuckGo)
            }
            ('b', "Bing", "https://www.bing.com/search?q={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::Bing)
            }
            ('y', "YouTube", "https://www.youtube.com/results?search_query={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::YouTube)
            }
            ('c', "ChatGpt", "https://chatgpt.com/?q={}") => {
                crate::io::WebSearch::Preset(crate::io::WebSearchPreset::ChatGpt)
            }
            _ => crate::io::WebSearch::Custom(crate::io::WebSearchCustom {
                key: value.key,
                name: value.name,
                url: value.url,
            }),
        }
    }
}

impl From<old_structs::Switch> for crate::io::Switch {
    fn from(value: old_structs::Switch) -> Self {
        Self {
            key: value.key,
            modifier: value.modifier,
            filter_by: value.filter_by,
            switch_workspaces: value.switch_workspaces,
            exclude_workspaces: value.exclude_workspaces,
            kill_key: value.kill_key,
            ..Default::default()
        }
    }
}
