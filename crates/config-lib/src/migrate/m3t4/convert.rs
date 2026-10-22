use crate::migrate::m3t4::{NEXT_CONFIG_VERSION, old_structs};
use crate::migrate::m4t5;

impl From<old_structs::Config> for m4t5::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            windows: value.windows.map(old_structs::Windows::into),
            version: NEXT_CONFIG_VERSION,
        }
    }
}

impl From<old_structs::Windows> for m4t5::Windows {
    fn from(value: old_structs::Windows) -> Self {
        Self {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switch: value.switch.map(old_structs::Switch::into),
            switch_2: value.switch_2.map(old_structs::Switch::into),
            overview: value.overview.map(old_structs::Overview::into),
        }
    }
}

impl From<old_structs::Overview> for m4t5::Overview {
    fn from(value: old_structs::Overview) -> Self {
        Self {
            key: value.key,
            modifier: value.modifier,
            filter_by: value.filter_by,
            launcher: value.launcher,
            exclude_workspaces: if value.exclude_special_workspaces.is_empty() {
                Box::from("")
            } else {
                format!("special:{}", value.exclude_special_workspaces).into_boxed_str()
            },
            ..Default::default()
        }
    }
}

impl From<old_structs::Switch> for m4t5::Switch {
    fn from(value: old_structs::Switch) -> Self {
        Self {
            key: value.key,
            modifier: value.modifier,
            filter_by: value.filter_by,
            switch_workspaces: value.switch_workspaces,
            exclude_workspaces: if value.exclude_special_workspaces.is_empty() {
                Box::from("")
            } else {
                format!("special:{}", value.exclude_special_workspaces).into_boxed_str()
            },
            kill_key: 'q',
        }
    }
}
