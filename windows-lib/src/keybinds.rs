use config_lib::Windows;
use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::transfer::{OpenSwitch, TransferType};

pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();
    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: vec![overview.modifier.to_str()],
            key: overview.key.clone(),

            exec: generate_transfer_socat(&TransferType::OpenOverview).into_boxed_str(),
        });
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("tab"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch { reverse: false }))
                .into_boxed_str(),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch { reverse: true }))
                .into_boxed_str(),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: Box::from("tab"),
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch { reverse: true }))
                .into_boxed_str(),
        });
    }

    binds
}
