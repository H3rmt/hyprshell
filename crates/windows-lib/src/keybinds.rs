use config_lib::{KeyMod, Windows};
use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::transfer::{HoldMod, OpenSwitch, TransferType};

#[must_use]
pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();
    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: vec![overview.modifier.to_str()],
            key: overview.key.clone(),
            exec: generate_transfer_socat(&TransferType::OpenOverview).into_boxed_str(),
        });
    }
    if let Some(switch) = windows.switches.first() {
        let base_mods = base_mods_from_switch(switch);
        if base_mods.is_empty() && switch.binds.forward.is_empty() {
            return binds;
        }
        let base_mods_str = base_mods.iter().map(|m| m.to_str()).collect::<Vec<_>>();
        let hold_mods = to_hold_mods(&base_mods);
        binds.push(ExecBind {
            mods: base_mods_str.clone(),
            key: Box::from("tab"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch {
                reverse: false,
                profile: 0,
                hold_mods: hold_mods.clone(),
            }))
            .into_boxed_str(),
        });
        binds.push(ExecBind {
            mods: base_mods_str.clone(),
            key: Box::from("grave"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                profile: 0,
                hold_mods: hold_mods.clone(),
            }))
            .into_boxed_str(),
        });
        let mut shift_mods_str = base_mods_str;
        shift_mods_str.push("shift");
        binds.push(ExecBind {
            mods: shift_mods_str,
            key: Box::from("tab"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                profile: 0,
                hold_mods,
            }))
            .into_boxed_str(),
        });
    }

    binds
}

fn base_mods_from_switch(switch: &config_lib::Switch) -> Vec<KeyMod> {
    switch
        .binds
        .forward
        .first()
        .map(|combo| {
            combo
                .mods
                .iter()
                .copied()
                .filter(|m| *m != KeyMod::Shift)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn to_hold_mods(mods: &[KeyMod]) -> Vec<HoldMod> {
    mods.iter()
        .filter_map(|m| match m {
            KeyMod::Alt => Some(HoldMod::Alt),
            KeyMod::Ctrl => Some(HoldMod::Ctrl),
            KeyMod::Super => Some(HoldMod::Super),
            KeyMod::Shift => None,
        })
        .collect()
}
