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
        });
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("tab"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: false,
            })),
            release: false,
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: Box::from("tab"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: format!("{}_l", switch.modifier.to_str()).into_boxed_str(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: format!("{}_r", switch.modifier.to_str()).into_boxed_str(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
        });
        binds.push(ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from("SHIFT_l"),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
        });
        binds.push(ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from("SHIFT_r"),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
        });
    }

    binds
}
