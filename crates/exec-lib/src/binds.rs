use core_lib::binds::{ExecBind, ExecBindBehavior};
use core_lib::{LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE, SWITCH_NAMESPACE};
use hyprland::bind_new::{Binding, Flag, Mod};
use hyprland::dispatch_new::Dispatch;
use hyprland::window_rule::{LayerEffect, LayerMatch, LayerRule};
use tracing::{trace, warn};

pub fn apply_layerrules() -> anyhow::Result<()> {
    // TODO add option to enable blur
    let rules = vec![
        LayerRule {
            name: None,
            r#match: vec![LayerMatch::Namespace(LAUNCHER_NAMESPACE.into())],
            effects: vec![LayerEffect::NoAnim(true), LayerEffect::Xray(false)],
        },
        LayerRule {
            name: None,
            r#match: vec![LayerMatch::Namespace(OVERVIEW_NAMESPACE.into())],
            effects: vec![LayerEffect::NoAnim(true), LayerEffect::Xray(false)],
        },
        LayerRule {
            name: None,
            r#match: vec![LayerMatch::Namespace(SWITCH_NAMESPACE.into())],
            effects: vec![
                LayerEffect::NoAnim(true),
                LayerEffect::Xray(false),
                LayerEffect::DimAround(true),
            ],
        },
    ];

    for rule in rules {
        rule.apply()?;
    }
    Ok(())
}

pub fn apply_exec_bind(bind: &ExecBind) -> anyhow::Result<()> {
    let binds: Vec<_> = bind
        .mods
        .iter()
        .filter_map(|m| match m.to_lowercase().as_str() {
            "alt" => Some(Mod::Alt),
            "control" | "ctrl" => Some(Mod::Ctrl),
            "super" | "win" => Some(Mod::Super),
            "shift" => Some(Mod::Shift),
            _ => {
                warn!("unknown mod: {m}");
                None
            }
        })
        .collect();
    let desc = format!("hyprshell-keybind: {}", bind.desc);

    let exec_prefix = if std::env::var_os("HYPRSHELL_ADD_KEYBIND_NOTIFICATIONS").is_some() {
        format!(
            "notify-send \"{} + {:?}\" \"{}\" -t 1000 -a hyprshell-settings && ",
            bind.key, binds, bind.desc
        )
    } else {
        String::new()
    };
    let exec = format!("{}{}", exec_prefix, bind.exec);

    let dispatcher = dispatcher_for_bind(bind, exec);

    let binding = Binding {
        mods: binds,
        key: bind.key.to_string(),
        flags: {
            let mut flags = vec![Flag::AutoConsuming, Flag::Description(desc)];
            match bind.behavior {
                ExecBindBehavior::Repeating => flags.push(Flag::Repeating),
                ExecBindBehavior::Release => flags.push(Flag::Release),
                ExecBindBehavior::TransparentReleaseIgnoringMods { .. } => {
                    // Do not swallow ordinary key presses just to watch for releases.
                    flags.extend([
                        Flag::Release,
                        Flag::Transparent,
                        Flag::IgnoreMods,
                        Flag::NonConsuming,
                    ]);
                }
            }
            flags
        },
        dispatcher,
    };
    trace!("binding exec: {binding:?}");
    binding.bind()?;
    Ok(())
}

fn dispatcher_for_bind(bind: &ExecBind, exec: String) -> Dispatch {
    let dispatcher = Dispatch::ExecCmd(exec, None);
    if let ExecBindBehavior::TransparentReleaseIgnoringMods {
        unless_keys_down: Some((left, right)),
    } = bind.behavior
    {
        // Evaluate in Hyprland at the key release, before spawning the socket command.
        // The Tab release must not close the switcher while Alt is still held.
        Dispatch::Unimplemented(format!(
            "function() if not hl.is_key_down(\"{left}\") and not hl.is_key_down(\"{right}\") then hl.dispatch({dispatcher}) end end"
        ))
    } else {
        dispatcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_release_checks_both_sides_before_dispatch() {
        let bind = ExecBind {
            mods: vec![],
            key: "Tab".into(),
            exec: String::new(),
            behavior: ExecBindBehavior::TransparentReleaseIgnoringMods {
                unless_keys_down: Some(("Alt_L", "Alt_R")),
            },
            desc: String::new(),
        };
        assert_eq!(
            dispatcher_for_bind(&bind, "echo close".into()).to_string(),
            "function() if not hl.is_key_down(\"Alt_L\") and not hl.is_key_down(\"Alt_R\") then hl.dispatch(hl.dsp.exec_cmd(\"echo close\")) end end"
        );
    }
}
