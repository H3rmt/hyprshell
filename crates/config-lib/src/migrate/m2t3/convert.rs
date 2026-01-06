use crate::migrate::m2t3::{NEXT_CONFIG_VERSION, old_structs};

impl From<old_structs::Config> for crate::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            windows: value.windows.map(old_structs::Windows::into),
            version: NEXT_CONFIG_VERSION,
        }
    }
}

impl From<old_structs::Windows> for crate::Windows {
    fn from(value: old_structs::Windows) -> Self {
        Self {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switch: value.switch,
            switch_2: None,
            overview: value.overview.map(old_structs::Overview::into),
        }
    }
}

impl From<old_structs::Overview> for crate::Overview {
    fn from(value: old_structs::Overview) -> Self {
        Self {
            key: value.key,
            modifier: value.modifier,
            filter_by: value.filter_by,
            hide_filtered: value.hide_filtered,
            launcher: value.launcher.into(),
            exclude_special_workspaces: "".into(),
        }
    }
}

impl From<old_structs::Launcher> for crate::Launcher {
    fn from(value: old_structs::Launcher) -> Self {
        Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier,
            width: value.width,
            show_when_empty: value.show_when_empty,
            max_items: value.max_items,
            plugins: value.plugins.into(),
        }
    }
}

impl From<old_structs::Plugins> for crate::Plugins {
    fn from(value: old_structs::Plugins) -> Self {
        Self {
            applications: value.applications,
            terminal: value.terminal,
            shell: value.shell,
            websearch: value.websearch,
            calc: value.calc,
            path: value.path,
            actions: Some(crate::ActionsPluginConfig::default()),
        }
    }
}
