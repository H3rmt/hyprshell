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
            timestamped: false,
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
                event_time: None,
                event_id: None,
            })),
            release: false,
            timestamped: true,
            desc: format!("Open Switch with {} + {}", switch.modifier, switch.key),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                event_time: None,
                event_id: None,
            })),
            release: false,
            timestamped: true,
            desc: format!("Open Switch (reverse) with {} + `", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                event_time: None,
                event_id: None,
            })),
            release: false,
            timestamped: true,
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
            timestamped: false,
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
            timestamped: false,
            desc: format!(
                "Close Switch (reverse) with {} + {}_r",
                switch.modifier, switch.modifier,
            ),
        });
    }

    binds
}

#[cfg(test)]
mod tests {
    use super::*;
    use config_lib::{Modifier, Switch};

    #[test]
    fn release_bindings_only_use_both_configured_modifier_sides() {
        for modifier in [Modifier::Alt, Modifier::Ctrl, Modifier::Super] {
            let windows = Windows {
                switch: Some(Switch {
                    modifier,
                    key: "F6".into(),
                    ..Switch::default()
                }),
                ..Windows::default()
            };
            let bindings = generate_open_keybinds(&windows);
            let releases: Vec<_> = bindings.iter().filter(|b| b.release).collect();
            assert_eq!(releases.len(), 2);
            assert_eq!(releases[0].key.as_ref(), modifier.to_keysym_l());
            assert_eq!(releases[1].key.as_ref(), modifier.to_keysym_r());
            assert!(
                releases
                    .iter()
                    .all(|b| !b.timestamped && b.exec.contains("\"switch\":true"))
            );
            let opens: Vec<_> = bindings.iter().filter(|b| !b.release).collect();
            assert_eq!(opens.len(), 3);
            assert!(opens.iter().all(|b| b.timestamped));
            assert_eq!(opens[0].key.as_ref(), "F6");
            assert_eq!(opens[2].mods, vec![modifier.to_str(), "shift"]);
            assert!(opens[2].exec.contains("\"reverse\":true"));
        }
    }
}
