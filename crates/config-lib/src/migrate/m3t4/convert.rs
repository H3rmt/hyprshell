use crate::migrate::m3t4::{NEXT_CONFIG_VERSION, old_structs};
use crate::{KeyCombo, KeyMod, Switch, SwitchBinds};

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
        let mut switches = Vec::new();
        if let Some(switch) = value.switch {
            switches.push(convert_switch(switch));
        }
        if let Some(switch) = value.switch_2 {
            switches.push(convert_switch(switch));
        }
        Self {
            scale: value.scale,
            items_per_row: value.items_per_row,
            switches,
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
            launcher: value.launcher,
            exclude_workspaces: value.exclude_special_workspaces,
        }
    }
}

fn convert_switch(value: old_structs::Switch) -> Switch {
    let base_mods = modifier_to_keymods(value.modifier);
    let forward = vec![KeyCombo {
        mods: base_mods.clone(),
        key: value.key,
        hold_mods: None,
    }];
    let mut reverse_mods = base_mods.clone();
    reverse_mods.push(KeyMod::Shift);
    let reverse = vec![
        KeyCombo {
            mods: reverse_mods,
            key: Box::from("Tab"),
            hold_mods: None,
        },
        KeyCombo {
            mods: base_mods,
            key: Box::from("grave"),
            hold_mods: None,
        },
    ];
    Switch {
        binds: SwitchBinds { forward, reverse },
        filter_by: value.filter_by,
        switch_workspaces: value.switch_workspaces,
        exclude_workspaces: value.exclude_special_workspaces,
        kill_key: value.kill_key,
    }
}

fn modifier_to_keymods(value: crate::Modifier) -> Vec<KeyMod> {
    match value {
        crate::Modifier::Alt => vec![KeyMod::Alt],
        crate::Modifier::Ctrl => vec![KeyMod::Ctrl],
        crate::Modifier::Super => vec![KeyMod::Super],
        crate::Modifier::None => Vec::new(),
    }
}
