use config_lib::Windows;
use core_lib::binds::{ExecBind, ExecBindBehavior, generate_transfer_socat};
use core_lib::transfer::{CloseSwitch, ExternalTransferType, OpenSwitch};

#[must_use]
pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();

    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: vec![overview.modifier.to_str()],
            key: overview.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenOverview),
            behavior: ExecBindBehavior::Release,
            desc: format!(
                "Open Overview with {} + {}",
                overview.modifier, overview.key
            ),
        });
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: false,
            })),
            behavior: ExecBindBehavior::Repeating,
            desc: format!("Open Switch with {} + {}", switch.modifier, switch.key),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            behavior: ExecBindBehavior::Repeating,
            desc: format!("Open Switch (reverse) with {} + `", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            behavior: ExecBindBehavior::Repeating,
            desc: format!(
                "Open Switch (reverse) with {} + shift + {}",
                switch.modifier, switch.key
            ),
        });

        binds.push(ExecBind {
            mods: vec![],
            key: switch.modifier.to_keysym_l().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            // The modifier mask at release can differ from the mask at press
            // (e.g. Tab was already held before Alt). Match the keysym instead.
            behavior: ExecBindBehavior::TransparentReleaseIgnoringMods {
                unless_keys_down: None,
            },
            desc: format!("Close Switch on {}_l release", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![],
            key: switch.modifier.to_keysym_r().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            behavior: ExecBindBehavior::TransparentReleaseIgnoringMods {
                unless_keys_down: None,
            },
            desc: format!("Close Switch on {}_r release", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            behavior: ExecBindBehavior::TransparentReleaseIgnoringMods {
                unless_keys_down: Some((
                    switch.modifier.to_keysym_l(),
                    switch.modifier.to_keysym_r(),
                )),
            },
            desc: format!(
                "Close Switch on {} release if {} is up",
                switch.key, switch.modifier
            ),
        });
    }

    binds
}
