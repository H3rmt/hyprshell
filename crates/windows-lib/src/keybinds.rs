use config_lib::{Modifier, Switch, Windows};
use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::transfer::{CloseSwitch, ExternalTransferType, OpenSwitch};

/// The binds that open one configured switcher.
///
/// `second` says which switcher the daemon should open, and is carried through
/// to [`OpenSwitch`]. `grave_reverse` controls the extra `<modifier> + grave`
/// opener: it is only a shorthand for `<modifier> + shift + <key>`, so the
/// caller drops it whenever that combination is spoken for.
fn open_binds(switch: &Switch, second: bool, grave_reverse: bool) -> Vec<ExecBind> {
    let name = if second { "Switch 2" } else { "Switch" };
    let mut binds = vec![
        ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: false,
                second,
            })),
            release: false,
            desc: format!("Open {name} with {} + {}", switch.modifier, switch.key),
        },
        ExecBind {
            mods: vec![switch.modifier.to_str(), "shift"],
            key: switch.key.clone(),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                second,
            })),
            release: false,
            desc: format!(
                "Open {name} (reverse) with {} + shift + {}",
                switch.modifier, switch.key
            ),
        },
    ];
    if grave_reverse {
        binds.push(ExecBind {
            mods: vec![switch.modifier.to_str()],
            key: Box::from("grave"),
            exec: generate_transfer_socat(&ExternalTransferType::OpenSwitch(OpenSwitch {
                reverse: true,
                second,
            })),
            release: false,
            desc: format!("Open {name} (reverse) with {} + `", switch.modifier),
        });
    }
    binds
}

/// The binds that close whichever switcher is open, when the held modifier goes up.
///
/// These key off the modifier rather than off a particular switcher, so two
/// switchers sharing a modifier need only one set between them.
fn close_binds(modifier: Modifier) -> Vec<ExecBind> {
    vec![
        ExecBind {
            mods: vec![modifier.to_str()],
            key: modifier.to_keysym_l().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: format!("Close Switch (reverse) with {modifier} + {modifier}_l"),
        },
        ExecBind {
            mods: vec![modifier.to_str()],
            key: modifier.to_keysym_r().into(),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: format!("Close Switch (reverse) with {modifier} + {modifier}_r"),
        },
    ]
}

/// Releasing shift also commits, for the `<modifier> + shift + <key>` direction.
fn shift_close_binds() -> Vec<ExecBind> {
    ["Shift_L", "Shift_R"]
        .into_iter()
        .map(|key| ExecBind {
            mods: vec!["SHIFT"],
            key: Box::from(key),
            exec: generate_transfer_socat(&ExternalTransferType::CloseSwitch(CloseSwitch {
                switch: true,
            })),
            release: true,
            desc: format!("Close Switch (reverse) with shift + {}", key.to_lowercase()),
        })
        .collect()
}

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

    let switches = [(&windows.switch, false), (&windows.switch_2, true)];

    // `<modifier> + grave` is only a shorthand for the reverse direction, so it
    // has to give way to a switcher actually configured on that key -- Hyprland
    // runs *every* bind matching a combo, so leaving both in place would open
    // the switcher twice over, in opposite directions.
    let grave_is_a_switcher_key = |modifier: Modifier| {
        switches
            .iter()
            .filter_map(|(switch, _)| switch.as_ref())
            .any(|s| s.modifier == modifier && s.key.eq_ignore_ascii_case("grave"))
    };

    // The shorthand and the close binds are per-modifier, not per-switcher, so
    // two switchers sharing a modifier must not each register their own.
    let mut grave_done: Vec<Modifier> = Vec::new();
    let mut close_done: Vec<Modifier> = Vec::new();

    for (switch, second) in switches {
        let Some(switch) = switch else { continue };

        let grave_reverse =
            !grave_is_a_switcher_key(switch.modifier) && !grave_done.contains(&switch.modifier);
        if grave_reverse {
            grave_done.push(switch.modifier);
        }
        binds.extend(open_binds(switch, second, grave_reverse));

        if !close_done.contains(&switch.modifier) {
            close_done.push(switch.modifier);
            binds.extend(close_binds(switch.modifier));
        }
    }

    if !close_done.is_empty() {
        binds.extend(shift_close_binds());
    }

    binds
}
