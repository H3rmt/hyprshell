use config_lib::Windows;
use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::transfer::{CloseSwitch, ExternalTransferType, OpenSwitch};

#[must_use]
pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();
    if let Some(overview) = &windows.overview {
        binds.push(generate_overview_binding(overview));
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: false,
            })),
            release: false,
            repeating: true,
            transparent: false,
            desc: format!("Open Switch with {} + {}", switch.modifier, switch.key),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
            repeating: true,
            transparent: false,
            desc: format!("Open Switch (reverse) with {} + `", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            release: false,
            repeating: true,
            transparent: false,
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
            repeating: false,
            transparent: true,
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
            repeating: false,
            transparent: true,
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
            repeating: false,
            transparent: true,
            desc: "Close Switch (reverse) with shift + shift_l".to_string(),
        });
        binds.push(ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from("Shift_R"),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            repeating: false,
            transparent: true,
            desc: "Close Switch (reverse) with shift + shift_r".to_string(),
        });
    }

    binds
}

fn generate_overview_binding(overview: &config_lib::Overview) -> ExecBind {
    ExecBind {
        mods: vec![overview.modifier.to_str()],
        key: overview.key.clone(),
        exec: generate_transfer_socat(&ExternalTransferType::OpenOverview),
        release: is_modifier_key(&overview.key),
        repeating: false,
        transparent: false,
        desc: format!(
            "Open Overview with {} + {}",
            overview.modifier, overview.key
        ),
    }
}

fn is_modifier_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "super_l"
            | "super_r"
            | "alt_l"
            | "alt_r"
            | "control_l"
            | "control_r"
            | "shift_l"
            | "shift_r"
            | "meta_l"
            | "meta_r"
            | "hyper_l"
            | "hyper_r"
            | "iso_level3_shift"
            | "iso_level5_shift"
            | "mode_switch"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use config_lib::{Modifier, Overview, Switch};

    fn overview_binding(key: &str, modifier: Modifier) -> ExecBind {
        let windows = Windows {
            overview: Some(Overview {
                key: key.into(),
                modifier,
                ..Default::default()
            }),
            switch: None,
            ..Default::default()
        };
        let mut bindings = generate_open_keybinds(&windows);
        assert_eq!(bindings.len(), 1);
        bindings.remove(0)
    }

    #[test]
    fn ordinary_overview_shortcuts_open_once_on_press() {
        for modifier in [Modifier::Alt, Modifier::Ctrl, Modifier::Super] {
            for key in ["Tab", "space", "Return", "a", "F6", "Left", "dead_grave"] {
                let binding = overview_binding(key, modifier);
                assert_eq!(binding.key.as_ref(), key);
                assert_eq!(binding.mods, [modifier.to_str()]);
                assert!(!binding.release, "{modifier} + {key} must open on press");
                assert!(
                    !binding.repeating,
                    "{modifier} + {key} must not toggle on repeat"
                );
                assert!(!binding.transparent);
            }
        }
    }

    #[test]
    fn modifier_overview_shortcuts_open_once_on_release() {
        for key in [
            "Super_L",
            "Super_R",
            "Alt_L",
            "Alt_R",
            "Control_L",
            "Control_R",
            "Shift_L",
            "Shift_R",
            "Meta_L",
            "Meta_R",
            "Hyper_L",
            "Hyper_R",
            "ISO_Level3_Shift",
            "ISO_Level5_Shift",
            "Mode_switch",
            "sUpEr_L",
        ] {
            let binding = overview_binding(key, Modifier::Super);
            assert!(binding.release, "{key} must wait for release");
            assert!(!binding.repeating);
            assert!(
                !binding.transparent,
                "{key} must not also fire after another shortcut"
            );
        }
        let default_overview = Overview::default();
        let binding = overview_binding(&default_overview.key, default_overview.modifier);
        assert!(binding.release);
        assert!(!binding.repeating && !binding.transparent);
    }

    #[test]
    fn switch_navigation_and_modifier_releases_keep_their_flags() {
        for modifier in [Modifier::Alt, Modifier::Ctrl, Modifier::Super] {
            let windows = Windows {
                overview: None,
                switch: Some(Switch {
                    modifier,
                    ..Default::default()
                }),
                ..Default::default()
            };
            let bindings = generate_open_keybinds(&windows);
            assert_eq!(bindings.len(), 7);
            assert_eq!(
                bindings.iter().filter(|binding| binding.repeating).count(),
                3
            );
            assert!(
                bindings
                    .iter()
                    .any(|binding| binding.key.as_ref() == "grave" && binding.repeating)
            );
            for binding in &bindings[..3] {
                assert!(!binding.release && binding.repeating && !binding.transparent);
            }
            let release_keys = [
                modifier.to_keysym_l(),
                modifier.to_keysym_r(),
                "Shift_L",
                "Shift_R",
            ];
            for (binding, expected_key) in bindings[3..].iter().zip(release_keys) {
                assert_eq!(binding.key.as_ref(), expected_key);
                assert!(binding.release && !binding.repeating && binding.transparent);
            }
        }
    }
}
