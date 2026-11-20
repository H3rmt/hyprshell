use config_lib::Windows;
use core_lib::binds::{ExecBind, ExecBindBehavior, Mod, generate_transfer_socat};
use core_lib::transfer::{CloseSwitch, ExternalTransferType, OpenSwitch};

#[must_use]
pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();

    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: if let Some(m) = overview.modifier {
                vec![m.into()]
            } else {
                vec![]
            },
            key: overview.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenOverview),
            behavior: ExecBindBehavior::Release,
            desc: if let Some(m) = overview.modifier {
                format!("Open Overview with {} + {}", m, overview.key)
            } else {
                format!("Open Overview with {}", overview.key)
            },
        });
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier.into()],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: false,
            })),
            behavior: ExecBindBehavior::Repeating,
            desc: format!("Open Switch with {} + {}", switch.modifier, switch.key),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.into()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
            })),
            behavior: ExecBindBehavior::Repeating,
            desc: format!("Open Switch (reverse) with {} + `", switch.modifier),
        });
        binds.push(ExecBind {
            mods: vec![switch.modifier.into(), Mod::Shift],
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

#[cfg(test)]
mod tests {
    use super::*;
    use config_lib::Switch;
    use core_lib::Modifier;

    #[test]
    fn modifier_release_binds_ignore_modifiers() {
        let mut windows = Windows::default();
        windows.switch = Some(Switch::default());
        let switch = windows.switch.as_mut().unwrap();
        switch.modifier = Modifier::Alt;
        switch.key = "Tab".into();

        let binds = generate_open_keybinds(&windows);
        for key in ["Alt_L", "Alt_R"] {
            let bind = binds.iter().find(|bind| bind.key.as_ref() == key).unwrap();
            assert_eq!(
                bind.behavior,
                ExecBindBehavior::TransparentReleaseIgnoringMods {
                    unless_keys_down: None
                }
            );
            assert!(bind.mods.is_empty());
        }

        let open = binds
            .iter()
            .find(|bind| bind.key.as_ref() == "Tab" && bind.behavior == ExecBindBehavior::Repeating)
            .unwrap();
        assert_eq!(open.mods, [Mod::Alt]);
        assert!(binds.iter().any(|bind| {
            bind.key.as_ref() == "Tab"
                && bind.mods == [Mod::Alt, Mod::Shift]
                && bind.behavior == ExecBindBehavior::Repeating
        }));

        let tab_release = binds
            .iter()
            .find(|bind| {
                bind.key.as_ref() == "Tab"
                    && matches!(
                        bind.behavior,
                        ExecBindBehavior::TransparentReleaseIgnoringMods { .. }
                    )
            })
            .unwrap();
        assert_eq!(
            tab_release.behavior,
            ExecBindBehavior::TransparentReleaseIgnoringMods {
                unless_keys_down: Some(("Alt_L", "Alt_R"))
            }
        );
        assert!(
            !binds
                .iter()
                .any(|bind| { matches!(bind.key.as_ref(), "Shift_L" | "Shift_R") })
        );
    }
}
