#[allow(clippy::wildcard_imports)]
use crate::io::*;

impl TryFrom<Config> for crate::Config {
    type Error = anyhow::Error;

    fn try_from(value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            windows: value.windows.map(Windows::try_into).transpose()?,
        })
    }
}

impl From<crate::Config> for Config {
    fn from(value: crate::Config) -> Self {
        Self {
            version: crate::CURRENT_CONFIG_VERSION,
            windows: value.windows.map(crate::Windows::into),
        }
    }
}

impl TryFrom<Windows> for crate::Windows {
    type Error = anyhow::Error;
    fn try_from(value: Windows) -> Result<Self, Self::Error> {
        Ok(Self {
            general: crate::WindowsGeneral {
                scale: value.scale,
                items_per_row: value.items_per_row,
            },
            switch: value.switch.map(Switch::try_into).transpose()?,
            switch_2: value.switch_2.map(Switch::try_into).transpose()?,
            overview: value.overview.map(Overview::try_into).transpose()?,
        })
    }
}

impl From<crate::Windows> for Windows {
    fn from(value: crate::Windows) -> Self {
        Self {
            scale: value.general.scale,
            items_per_row: value.general.items_per_row,
            overview: value.overview.map(crate::Overview::into),
            switch: value.switch.map(crate::Switch::into),
            switch_2: value.switch_2.map(crate::Switch::into),
        }
    }
}

impl TryFrom<Overview> for crate::Overview {
    type Error = anyhow::Error;
    fn try_from(value: Overview) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            modifier: value.modifier,
            top_offset: value.top_offset,
            filter_by_same_class: value.filter_by.contains(&FilterBy::SameClass),
            filter_by_current_workspace: value.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_by_current_monitor: value.filter_by.contains(&FilterBy::CurrentMonitor),
            launcher: value.launcher.try_into()?,
            exclude_workspaces: value.exclude_workspaces,
        })
    }
}

impl From<crate::Overview> for Overview {
    fn from(value: crate::Overview) -> Self {
        let mut filter = vec![];
        if value.filter_by_current_monitor {
            filter.push(FilterBy::CurrentMonitor);
        }
        if value.filter_by_current_workspace {
            filter.push(FilterBy::CurrentWorkspace);
        }
        if value.filter_by_same_class {
            filter.push(FilterBy::SameClass);
        }
        Self {
            launcher: value.launcher.into(),
            key: value.key,
            modifier: value.modifier,
            filter_by: filter,
            exclude_workspaces: value.exclude_workspaces,
            top_offset: value.top_offset,
        }
    }
}

impl TryFrom<Switch> for crate::Switch {
    type Error = anyhow::Error;
    fn try_from(value: Switch) -> Result<Self, Self::Error> {
        // TODO check key +  kill key
        Ok(Self {
            modifier: value.modifier,
            key: value.key,
            filter_by_same_class: value.filter_by.contains(&FilterBy::SameClass),
            filter_by_current_workspace: value.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_by_current_monitor: value.filter_by.contains(&FilterBy::CurrentMonitor),
            switch_workspaces: value.switch_workspaces,
            exclude_workspaces: value.exclude_workspaces,
            kill_key: value.kill_key,
        })
    }
}

impl From<crate::Switch> for Switch {
    fn from(value: crate::Switch) -> Self {
        let mut filter = vec![];
        if value.filter_by_current_monitor {
            filter.push(FilterBy::CurrentMonitor);
        }
        if value.filter_by_current_workspace {
            filter.push(FilterBy::CurrentWorkspace);
        }
        if value.filter_by_same_class {
            filter.push(FilterBy::SameClass);
        }
        Self {
            modifier: value.modifier,
            key: value.key,
            filter_by: filter,
            switch_workspaces: value.switch_workspaces,
            exclude_workspaces: value.exclude_workspaces,
            kill_key: value.kill_key,
        }
    }
}

impl TryFrom<Launcher> for crate::Launcher {
    type Error = anyhow::Error;
    fn try_from(value: Launcher) -> Result<Self, Self::Error> {
        Ok(Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier,
            width: value.width,
            show_when_empty: value.show_when_empty,
            max_items: value.max_items,
            plugins: value.plugins.try_into()?,
        })
    }
}

impl From<crate::Launcher> for Launcher {
    fn from(value: crate::Launcher) -> Self {
        Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier,
            width: value.width,
            max_items: value.max_items,
            show_when_empty: value.show_when_empty,
            plugins: value.plugins.into(),
        }
    }
}

impl TryFrom<Plugins> for crate::Plugins {
    type Error = anyhow::Error;

    fn try_from(value: Plugins) -> Result<Self, Self::Error> {
        Ok(Self {
            applications: value
                .applications
                .map(ApplicationsPluginConfig::try_into)
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
            websearch: value.websearch.map(WebSearchConfig::try_into).transpose()?,
            calc: value.calc.map(CalcPluginConfig::try_into).transpose()?,
            path: if value.path.is_some() { Some(()) } else { None },
            actions: value
                .actions
                .map(ActionsPluginConfig::try_into)
                .transpose()?,
        })
    }
}

impl From<crate::Plugins> for Plugins {
    fn from(value: crate::Plugins) -> Self {
        Self {
            applications: value.applications.map(Into::into),
            terminal: if value.terminal.is_some() {
                Some(EmptyConfig {})
            } else {
                None
            },
            shell: if value.shell.is_some() {
                Some(EmptyConfig {})
            } else {
                None
            },
            websearch: value.websearch.map(Into::into),
            calc: value.calc.map(Into::into),
            path: if value.path.is_some() {
                Some(EmptyConfig {})
            } else {
                None
            },
            actions: value.actions.map(Into::into),
        }
    }
}

impl TryFrom<ActionsPluginConfig> for crate::ActionsPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: ActionsPluginConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            actions: value
                .actions
                .into_iter()
                .map(ActionsPluginAction::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<crate::ActionsPluginConfig> for ActionsPluginConfig {
    fn from(value: crate::ActionsPluginConfig) -> Self {
        Self {
            actions: value.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<ApplicationsPluginConfig> for crate::ApplicationsPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: ApplicationsPluginConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            run_cache_weeks: value.run_cache_weeks,
            show_execs: value.show_execs,
            show_actions_submenu: value.show_actions_submenu,
        })
    }
}

impl From<crate::ApplicationsPluginConfig> for ApplicationsPluginConfig {
    fn from(value: crate::ApplicationsPluginConfig) -> Self {
        Self {
            run_cache_weeks: value.run_cache_weeks,
            show_execs: value.show_execs,
            show_actions_submenu: value.show_actions_submenu,
        }
    }
}

impl TryFrom<WebSearchConfig> for crate::WebSearchConfig {
    type Error = anyhow::Error;

    fn try_from(value: WebSearchConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            engines: value
                .engines
                .into_iter()
                .map(SearchEngine::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<crate::WebSearchConfig> for WebSearchConfig {
    fn from(value: crate::WebSearchConfig) -> Self {
        Self {
            engines: value.engines.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<SearchEngine> for crate::SearchEngine {
    type Error = anyhow::Error;

    fn try_from(value: SearchEngine) -> Result<Self, Self::Error> {
        Ok(Self {
            url: value.url,
            name: value.name,
            key: value.key,
        })
    }
}

impl From<crate::SearchEngine> for SearchEngine {
    fn from(value: crate::SearchEngine) -> Self {
        Self {
            url: value.url,
            name: value.name,
            key: value.key,
        }
    }
}

impl TryFrom<ActionsPluginAction> for crate::ActionsPluginAction {
    type Error = anyhow::Error;

    fn try_from(value: ActionsPluginAction) -> Result<Self, Self::Error> {
        match value {
            ActionsPluginAction::LockScreen => Ok(Self::LockScreen),
            ActionsPluginAction::Hibernate => Ok(Self::Hibernate),
            ActionsPluginAction::Logout => Ok(Self::Logout),
            ActionsPluginAction::Reboot => Ok(Self::Reboot),
            ActionsPluginAction::Shutdown => Ok(Self::Shutdown),
            ActionsPluginAction::Suspend => Ok(Self::Suspend),
            ActionsPluginAction::Custom(v) => Ok(Self::Custom(v.try_into()?)),
        }
    }
}

impl From<crate::ActionsPluginAction> for ActionsPluginAction {
    fn from(value: crate::ActionsPluginAction) -> Self {
        match value {
            crate::ActionsPluginAction::LockScreen => Self::LockScreen,
            crate::ActionsPluginAction::Hibernate => Self::Hibernate,
            crate::ActionsPluginAction::Logout => Self::Logout,
            crate::ActionsPluginAction::Reboot => Self::Reboot,
            crate::ActionsPluginAction::Shutdown => Self::Shutdown,
            crate::ActionsPluginAction::Suspend => Self::Suspend,
            crate::ActionsPluginAction::Custom(v) => Self::Custom(v.into()),
        }
    }
}

impl TryFrom<ActionsPluginActionCustom> for crate::ActionsPluginActionCustom {
    type Error = anyhow::Error;

    fn try_from(value: ActionsPluginActionCustom) -> Result<Self, Self::Error> {
        // TODO check icon
        Ok(Self {
            names: value.names,
            details: value.details,
            command: value.command,
            icon: value.icon,
        })
    }
}

impl From<crate::ActionsPluginActionCustom> for ActionsPluginActionCustom {
    fn from(value: crate::ActionsPluginActionCustom) -> Self {
        Self {
            names: value.names,
            details: value.details,
            command: value.command,
            icon: value.icon,
        }
    }
}

impl TryFrom<CalcPluginConfig> for crate::CalcPluginConfig {
    type Error = anyhow::Error;

    fn try_from(value: CalcPluginConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            prefix: value.prefix,
        })
    }
}

impl From<crate::CalcPluginConfig> for CalcPluginConfig {
    fn from(value: crate::CalcPluginConfig) -> Self {
        Self {
            prefix: value.prefix,
        }
    }
}
