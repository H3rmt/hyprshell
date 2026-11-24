use std::{collections::HashSet, path::PathBuf};

use anyhow::bail;

#[allow(clippy::wildcard_imports)]
use crate::structs::*;

use crate::io;

impl TryFrom<io::Config> for Config {
    type Error = anyhow::Error;

    fn try_from(value: io::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            windows: value.windows.map(io::Windows::try_into).transpose()?,
        })
    }
}

impl TryFrom<io::Windows> for crate::Windows {
    type Error = anyhow::Error;
    fn try_from(value: io::Windows) -> Result<Self, Self::Error> {
        if value.scale >= 20f64 || value.scale <= 0f64 {
            bail!("Scale factor must be less than 20 and greater than 0");
        }
        Ok(Self {
            general: crate::WindowsGeneral {
                scale: value.scale,
                items_per_row: value.items_per_row,
                live_preview_refresh_rate: value.live_settings.refresh_rate,
            },
            switch: value.switch.map(io::Switch::try_into).transpose()?,
            switch_2: value.switch_2.map(io::Switch::try_into).transpose()?,
            overview: value.overview.map(io::Overview::try_into).transpose()?,
        })
    }
}

impl TryFrom<io::Overview> for crate::Overview {
    type Error = anyhow::Error;
    fn try_from(value: io::Overview) -> Result<Self, Self::Error> {
        if ["super", "alt", "ctrl", "control"].contains(&&*value.key) {
            bail!(
                "If a modifier key is used to open it must include _l or _r at the end. (e.g. super_l, alt_r, etc)\nctrl_l / _r is NOT a valid modifier key, only control_l / _r is"
            );
        }
        if value.filter_by.contains(&io::FilterBy::CurrentWorkspace)
            && value.filter_by.contains(&io::FilterBy::CurrentMonitor)
        {
            bail!("Overview cant filter by CurrentWorkspace and CurrentMonitor at the same time")
        }
        Ok(Self {
            key: value.key,
            modifier: value.modifier.into(),
            top_offset: value.top_offset,
            filter_by_same_class: value.filter_by.contains(&io::FilterBy::SameClass),
            filter_by_current_workspace: value.filter_by.contains(&io::FilterBy::CurrentWorkspace),
            filter_by_current_monitor: value.filter_by.contains(&io::FilterBy::CurrentMonitor),
            launcher: value.launcher.try_into()?,
            exclude_workspaces: value.exclude_workspaces,
            live_preview: value.live_preview,
        })
    }
}

impl TryFrom<io::Switch> for crate::Switch {
    type Error = anyhow::Error;
    fn try_from(value: io::Switch) -> Result<Self, Self::Error> {
        if value.filter_by.contains(&io::FilterBy::CurrentWorkspace)
            && value.filter_by.contains(&io::FilterBy::CurrentMonitor)
        {
            bail!("Switch cant filter by CurrentWorkspace and CurrentMonitor at the same time")
        }

        Ok(Self {
            modifier: value.modifier.try_into()?,
            key: value.key,
            filter_by_same_class: value.filter_by.contains(&io::FilterBy::SameClass),
            filter_by_current_workspace: value.filter_by.contains(&io::FilterBy::CurrentWorkspace),
            filter_by_current_monitor: value.filter_by.contains(&io::FilterBy::CurrentMonitor),
            switch_workspaces: value.switch_workspaces,
            exclude_workspaces: value.exclude_workspaces,
            kill_key: value.kill_key,
            live_preview: value.live_preview,
        })
    }
}

impl TryFrom<io::Launcher> for crate::Launcher {
    type Error = anyhow::Error;
    fn try_from(value: io::Launcher) -> Result<Self, Self::Error> {
        if value
            .default_terminal
            .as_ref()
            .is_some_and(|dt| dt.is_empty())
        {
            bail!(
                "Default terminal command cannot be empty string. Leave out / set to None in order for the first terminal in &PATH to be used"
            );
        }
        if value.width < 10 {
            bail!("Width of launcher must be greater than 10");
        }

        Ok(Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier.try_into()?,
            alt_launch_modifier: match value.launch_modifier {
                io::Modifier::Alt => core_lib::Modifier::Ctrl,
                _ => core_lib::Modifier::Alt,
            },
            width: value.width,
            show_when_empty: value.show_when_empty,
            max_items: value.max_items,
            plugins: value.plugins.try_into()?,
        })
    }
}

impl TryFrom<io::Plugins> for crate::Plugins {
    type Error = anyhow::Error;

    fn try_from(value: io::Plugins) -> Result<Self, Self::Error> {
        Ok(Self {
            applications: value
                .applications
                .map(io::ApplicationsPluginConfig::try_into)
                .transpose()?,
            terminal: if value.terminal.is_some() {
                Some(())
            } else {
                None
            },
            shell: if value.shell.is_some() {
                Some(())
            } else {
                None
            },
            websearch: value
                .websearch
                .map(io::WebSearchConfig::try_into)
                .transpose()?,
            calc: value.calc.map(io::CalcPluginConfig::try_into).transpose()?,
            path: if value.path.is_some() { Some(()) } else { None },
            actions: value
                .actions
                .map(io::ActionsPluginConfig::try_into)
                .transpose()?,
        })
    }
}

impl TryFrom<io::ActionsPluginConfig> for crate::ActionsPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: io::ActionsPluginConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            actions: value
                .actions
                .into_iter()
                .map(io::ActionsPluginAction::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
impl TryFrom<io::ApplicationsPluginConfig> for crate::ApplicationsPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: io::ApplicationsPluginConfig) -> Result<Self, Self::Error> {
        if !(value.run_cache_weeks > 0) {
            bail!("Number of weeks for Application run cache must be greater than 0")
        }

        Ok(Self {
            run_cache_weeks: value.run_cache_weeks,
        })
    }
}

impl TryFrom<io::WebSearchConfig> for crate::WebSearchConfig {
    type Error = anyhow::Error;

    fn try_from(value: io::WebSearchConfig) -> Result<Self, Self::Error> {
        let mut keys = HashSet::<char>::with_capacity(value.engines.len());
        let engines: Vec<crate::SearchEngine> = value
            .engines
            .into_iter()
            .map(io::WebSearch::try_into)
            .collect::<Result<_, _>>()?;

        for e in &engines {
            if keys.contains(&e.key) {
                bail!("Duplicate search engine key: {}", e.key);
            }
            keys.insert(e.key);
        }

        Ok(Self { engines })
    }
}

impl TryFrom<io::WebSearch> for crate::SearchEngine {
    type Error = anyhow::Error;

    fn try_from(value: io::WebSearch) -> Result<Self, Self::Error> {
        let value = match value {
            io::WebSearch::Preset(io::WebSearchPreset::Bing) => Self {
                name: Box::from("Bing"),
                key: 'b',
                url: Box::from("https://www.bing.com/search?q={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::Google) => Self {
                name: Box::from("Google"),
                key: 'g',
                url: Box::from("https://www.google.com/search?q={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::Startpage) => Self {
                name: Box::from("Startpage"),
                key: 's',
                url: Box::from("https://www.startpage.com/sp/search?query={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::DuckDuckGo) => Self {
                name: Box::from("DuckDuckGo"),
                key: 'd',
                url: Box::from("https://duckduckgo.com/?q={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::Wikipedia) => Self {
                name: Box::from("Wikipedia"),
                key: 'w',
                url: Box::from("https://en.wikipedia.org/wiki/Special:Search?search={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::ChatGpt) => Self {
                name: Box::from("ChatGPT"),
                key: 'c',
                url: Box::from("https://chatgpt.com/?q={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::YouTube) => Self {
                name: Box::from("YouTube"),
                key: 'y',
                url: Box::from("https://www.youtube.com/results?search_query={}"),
            },
            io::WebSearch::Preset(io::WebSearchPreset::Reddit) => Self {
                name: Box::from("Reddit"),
                key: 'r',
                url: Box::from("https://www.reddit.com/search?q={}"),
            },
            io::WebSearch::Custom(c) => Self {
                name: c.name.clone(),
                key: c.key,
                url: c.url,
            },
        };

        if !value.url.contains("{}") {
            tracing::warn!("{}s url doesnt contain {{}}: {}", value.name, value.url)
        }
        if value.url.is_empty() {
            bail!("Search engine url cannot be empty");
        }
        if value.name.is_empty() {
            bail!("Search engine name cannot be empty");
        }
        Ok(value)
    }
}

impl TryFrom<io::ActionsPluginAction> for crate::ActionsPluginAction {
    type Error = anyhow::Error;

    fn try_from(value: io::ActionsPluginAction) -> Result<Self, Self::Error> {
        let value = match value {
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::LockScreen) => Self {
                name: Box::from("Lock Screen"),
                keywords: Box::new([Box::from("user")]),
                details_long: Some(Box::from("Lock the screen")),
                command: Box::from("loginctl lock-session"),
                icon: Some(PathBuf::from("system-lock-screen").into_boxed_path()),
                children: Box::new([]),
            },
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::Hibernate) => Self {
                name: Box::from("Hibernate"),
                keywords: Box::new([]),
                details_long: Some(Box::from(
                    "Save your session to disk and power down. Resume it when you turn the computer on",
                )),
                command: Box::from("loginctl lock-session && systemctl hibernate"),
                icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                children: Box::new([]),
            },
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::Logout) => Self {
                name: Box::from("Log Out"),
                keywords: Box::new([Box::from("user")]),
                command: Box::from("loginctl terminate-session self"),
                details_long: Some(Box::from("Log out of the session")),
                icon: Some(PathBuf::from("system-log-out").into_boxed_path()),
                children: Box::new([]),
            },
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::Reboot) => Self {
                name: Box::from("Reboot / Restart"),
                keywords: Box::new([]),
                command: Box::from("systemctl reboot"),
                details_long: Some(Box::from("Reboot the computer")),
                icon: Some(PathBuf::from("system-reboot").into_boxed_path()),
                children: Box::new([]),
            },
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::Shutdown) => Self {
                name: Box::from("Shutdown / Poweroff"),
                keywords: Box::new([]),
                command: Box::from("systemctl poweroff"),
                details_long: Some(Box::from("Shut down the computer")),
                icon: Some(PathBuf::from("system-shutdown").into_boxed_path()),
                children: Box::new([]),
            },
            io::ActionsPluginAction::Preset(io::ActionsPluginActionPreset::Suspend) => Self {
                name: Box::from("Sleep"),
                keywords: Box::new([]),
                command: Box::from("loginctl lock-session && systemctl suspend"),
                details_long: Some(Box::from(
                    "Suspend to RAM for a quick wake. Requires power to preserve your session",
                )),
                icon: Some(PathBuf::from("system-suspend").into_boxed_path()),
                children: Box::new([
                    Self {
                        name: Box::from("Sleep, Then Hibernate"),
                        keywords: Box::new([]),
                        icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                        command: Box::from(
                            "loginctl lock-session && systemctl suspend-then-hibernate",
                        ),
                        details_long: Some(Box::from(
                            "Suspend now, then save your session to disk and power down later",
                        )),
                        children: Box::new([]),
                    },
                    Self {
                        name: Box::from("Hybrid Sleep"),
                        keywords: Box::new([]),
                        icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                        command: Box::from("loginctl lock-session && systemctl hybrid-sleep"),
                        details_long: Some(Box::from(
                            "Save your session to disk, then suspend for a quick wake",
                        )),
                        children: Box::new([]),
                    },
                ]),
            },
            io::ActionsPluginAction::Custom(value) => Self {
                name: value.name,
                keywords: Box::from([]),
                command: value.command,
                details_long: None,
                icon: value.icon,
                // implement this some day
                children: Box::from([]),
            },
        };

        if value.command.is_empty() {
            bail!("Action ({}) command cannot be empty", value.name);
        }
        if value.name.is_empty() {
            bail!("Action ({}) name cannot be empty", value.command);
        }
        Ok(value)
    }
}

impl TryFrom<io::CalcPluginConfig> for crate::CalcPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: io::CalcPluginConfig) -> Result<Self, Self::Error> {
        #[cfg(not(feature = "launcher_calc_plugin"))]
        {
            bail!(
                "Calc Plugin enabled but not compiled in but enabled, please enable the calc feature"
            );
        }

        Ok(Self {
            prefix: value.prefix,
        })
    }
}

impl From<io::Modifier> for Option<core_lib::Modifier> {
    fn from(value: io::Modifier) -> Self {
        match value {
            io::Modifier::Alt => Some(core_lib::Modifier::Alt),
            io::Modifier::Ctrl => Some(core_lib::Modifier::Ctrl),
            io::Modifier::Super => Some(core_lib::Modifier::Super),
            io::Modifier::None => None,
        }
    }
}

impl TryFrom<io::Modifier> for core_lib::Modifier {
    type Error = anyhow::Error;

    fn try_from(value: io::Modifier) -> Result<Self, Self::Error> {
        match value {
            io::Modifier::Alt => Ok(core_lib::Modifier::Alt),
            io::Modifier::Ctrl => Ok(core_lib::Modifier::Ctrl),
            io::Modifier::Super => Ok(core_lib::Modifier::Super),
            io::Modifier::None => bail!("none modifier not allowed"),
        }
    }
}
