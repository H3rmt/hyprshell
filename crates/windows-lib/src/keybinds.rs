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
        for key in switch.forward_keys() {
            binds.push(ExecBind {
                mods: vec![switch.modifier.to_str()],
                key: key.clone(),
                exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                    reverse: false,
                })),
                release: false,
                desc: format!("Open Switch with {} + {key}", switch.modifier),
            });
        }
        for key in &switch.reverse_keys {
            binds.push(ExecBind {
                mods: vec![switch.modifier.to_str()],
                key: key.clone(),
                exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                    reverse: true,
                })),
                release: false,
                desc: format!("Open Switch (reverse) with {} + {key}", switch.modifier),
            });
        }
        for key in switch.forward_keys() {
            binds.push(ExecBind {
                mods: vec![switch.modifier.to_str(), "shift"],
                key: key.clone(),
                exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                    reverse: true,
                })),
                release: false,
                desc: format!(
                    "Open Switch (reverse) with {} + shift + {key}",
                    switch.modifier
                ),
            });
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use config_lib::{Modifier, Switch};

    fn windows(switch: Switch) -> Windows {
        Windows {
            switch: Some(switch),
            ..Default::default()
        }
    }

    #[test]
    fn defaults_retain_all_existing_switch_chords() {
        let bindings = generate_open_keybinds(&windows(Switch::default()));
        assert_eq!(bindings.len(), 7);
        assert_eq!(bindings.iter().filter(|binding| binding.release).count(), 4);
        assert!(
            bindings
                .iter()
                .any(|binding| binding.key.as_ref() == "grave" && binding.exec.contains("true"))
        );
    }

    #[test]
    fn disabling_grave_preserves_tab_shift_tab_and_modifier_releases() {
        let bindings = generate_open_keybinds(&windows(Switch {
            modifier: Modifier::Ctrl,
            reverse_keys: Vec::new(),
            ..Default::default()
        }));
        assert_eq!(bindings.len(), 6);
        assert!(
            !bindings
                .iter()
                .any(|binding| binding.key.as_ref() == "grave")
        );
        assert!(bindings.iter().any(|binding| binding.mods == ["ctrl"]
            && binding.key.as_ref() == "Tab"
            && binding.exec.contains("false")));
        assert!(
            bindings
                .iter()
                .any(|binding| binding.mods == ["ctrl", "shift"]
                    && binding.key.as_ref() == "Tab"
                    && binding.exec.contains("true"))
        );
        for key in ["Control_L", "Control_R", "Shift_L", "Shift_R"] {
            assert_eq!(
                bindings
                    .iter()
                    .filter(|binding| binding.release && binding.key.as_ref() == key)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn multiple_keys_generate_each_direction_without_duplicate_releases() {
        let bindings = generate_open_keybinds(&windows(Switch {
            key: "F8".into(),
            keys: Some(vec!["Tab".into(), "F6".into()]),
            reverse_keys: vec!["grave".into(), "F7".into()],
            ..Default::default()
        }));
        assert_eq!(bindings.len(), 10);
        assert_eq!(bindings.iter().filter(|binding| binding.release).count(), 4);
        assert!(!bindings.iter().any(|binding| binding.key.as_ref() == "F8"));
        assert_eq!(
            bindings
                .iter()
                .filter(|binding| !binding.release && binding.exec.contains("false"))
                .count(),
            2
        );
        assert_eq!(
            bindings
                .iter()
                .filter(|binding| !binding.release && binding.exec.contains("true"))
                .count(),
            4
        );
    }

    #[test]
    fn overview_is_unchanged_and_switch_two_is_not_activated() {
        let mut windows = windows(Switch::default());
        windows.overview = Some(config_lib::Overview::default());
        let before = generate_open_keybinds(&windows);
        windows.switch_2 = Some(Switch {
            keys: Some(vec!["F9".into()]),
            ..Default::default()
        });
        let after = generate_open_keybinds(&windows);
        assert_eq!(before.len(), after.len());
        assert_eq!(before[0].exec, after[0].exec);
        assert_eq!(before[0].key, after[0].key);
        assert_eq!(before[0].mods, after[0].mods);
        assert!(!after.iter().any(|binding| binding.key.as_ref() == "F9"));
    }
}
