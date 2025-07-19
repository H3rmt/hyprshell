use crate::migrate::m1t2::old_structs;
use crate::{CURRENT_CONFIG_VERSION, Config, Launcher, Overview, Switch, Windows};

impl From<old_structs::Config> for Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            layerrules: value.layerrules,
            kill_bind: value.kill_bind.into(),
            windows: value.windows.map(old_structs::Windows::into),
            version: CURRENT_CONFIG_VERSION,
        }
    }
}

impl From<old_structs::Windows> for Windows {
    fn from(value: old_structs::Windows) -> Windows {
        Windows {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switch: value.switch.map(old_structs::Switch::into),
            overview: value.overview.map(old_structs::Overview::into),
        }
    }
}

impl From<old_structs::Overview> for Overview {
    fn from(value: old_structs::Overview) -> Overview {
        Overview {
            key: value.key,
            modifier: value.modifier.into(),
            filter_by: value.filter_by,
            hide_filtered: value.hide_filtered,
            launcher: value.launcher.into(),
        }
    }
}

impl From<old_structs::Switch> for Switch {
    fn from(value: old_structs::Switch) -> Switch {
        Switch {
            filter_by: value.filter_by,
            modifier: value.modifier.into(),
            show_workspaces: value.show_workspaces,
        }
    }
}

impl From<old_structs::Launcher> for Launcher {
    fn from(value: old_structs::Launcher) -> Launcher {
        let mut plugins = value.plugins;
        if let Some(a) = &mut plugins.applications {
            a.show_actions_submenu = true
        };
        plugins.path = Some(Default::default());
        Launcher {
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
            old_structs::Modifier::Alt => crate::Modifier::Alt,
            old_structs::Modifier::Ctrl => crate::Modifier::Ctrl,
            old_structs::Modifier::Shift => crate::Modifier::Shift,
            old_structs::Modifier::Super => crate::Modifier::Super,
        }
    }
}
