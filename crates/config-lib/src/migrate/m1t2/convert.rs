use crate::migrate::m1t2::{NEXT_CONFIG_VERSION, old_structs};
use crate::migrate::m2t3;

impl From<old_structs::Config> for m2t3::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            layerrules: value.layerrules,
            kill_bind: value.kill_bind,
            windows: value.windows.map(old_structs::Windows::into),
            version: Some(NEXT_CONFIG_VERSION),
        }
    }
}

impl From<old_structs::Windows> for m2t3::Windows {
    fn from(value: old_structs::Windows) -> Self {
        Self {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switch: value.switch.map(old_structs::Switch::into),
            overview: value.overview.map(old_structs::Overview::into),
        }
    }
}

impl From<old_structs::Overview> for m2t3::Overview {
    fn from(value: old_structs::Overview) -> Self {
        Self {
            key: value.key,
            modifier: value.modifier.into(),
            filter_by: value.filter_by,
            hide_filtered: value.hide_filtered,
            launcher: value.launcher.into(),
        }
    }
}

impl From<old_structs::Switch> for crate::Switch {
    fn from(value: old_structs::Switch) -> Self {
        Self {
            filter_by: value.filter_by,
            modifier: value.modifier.into(),
            key: "tab".into(),
            switch_workspaces: value.show_workspaces,
            exclude_special_workspaces: "".into(),
        }
    }
}

impl From<old_structs::Launcher> for m2t3::Launcher {
    fn from(value: old_structs::Launcher) -> Self {
        let mut plugins = value.plugins;
        if let Some(a) = &mut plugins.applications {
            a.show_actions_submenu = true;
        }
        plugins.path = Some(crate::EmptyConfig::default());
        Self {
            default_terminal: value.default_terminal,
            launch_modifier: value.launch_modifier.into(),
            width: value.width,
            show_when_empty: value.show_when_empty,
            max_items: value.max_items,
            plugins,
        }
    }
}

impl From<old_structs::Modifier> for crate::Modifier {
    fn from(value: old_structs::Modifier) -> Self {
        match value {
            old_structs::Modifier::Alt => Self::Alt,
            old_structs::Modifier::Ctrl => Self::Ctrl,
            old_structs::Modifier::Shift | old_structs::Modifier::Super => Self::Super, // Shift removed
        }
    }
}
