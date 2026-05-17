use config_lib::Windows;
use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::transfer::{CloseSwitch, ExternalTransferType, OpenSwitch};

#[must_use]
pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();
    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: vec![overview.modifier.to_str()],
            key: overview.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenOverview),
            release: false,
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
            release: false,
            desc: format!("Open Switch with {} + {}", switch.modifier, switch.key),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
            desc: format!("Open Switch (reverse) with {} + `", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
            desc: format!(
                "Open Switch (reverse) with {} + shift + {}",
                switch.modifier, switch.key
            ),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: switch.modifier.to_keysym_l().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: format!(
                "Close Switch (reverse) with {} + {}_l",
                switch.modifier, switch.modifier,
            ),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: switch.modifier.to_keysym_r().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: format!(
                "Close Switch (reverse) with {} + {}_r",
                switch.modifier, switch.modifier,
            ),
        });
        binds.push(ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from("Shift_L"),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: "Close Switch (reverse) with shift + shift_l".to_string(),
        });
        binds.push(ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from("Shift_R"),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: "Close Switch (reverse) with shift + shift_r".to_string(),
        });
    }

    binds
}
