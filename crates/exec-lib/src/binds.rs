use core_lib::binds::{ExecBind, ExecBindBehavior, Mod};
use core_lib::{LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE, SWITCH_NAMESPACE};
use hyprland::bind_new::{Binding, Flag, Mod as HyprlandMod};
use hyprland::data::Binds;
use hyprland::dispatch_new::Dispatch;
use hyprland::prelude::{HyprData, HyprDataVec};
use hyprland::window_rule::{LayerEffect, LayerMatch, LayerRule};
use tracing::trace;

const HYPRSHELL_BIND_DESCRIPTION_PREFIX: &str = "hyprshell-keybind: ";

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
    let desc = format!("{}{}", HYPRSHELL_BIND_DESCRIPTION_PREFIX, bind.desc);

    let exec_prefix = if std::env::var_os("HYPRSHELL_ADD_KEYBIND_NOTIFICATIONS").is_some() {
        format!(
            "notify-send \"{} + {:?}\" \"{}\" -t 1000 -a hyprshell-settings && ",
            bind.key, bind.mods, bind.desc
        )
    } else {
        String::new()
    };
    let exec = format!("{}{}", exec_prefix, bind.exec);

    let dispatcher = dispatcher_for_bind(bind, exec);

    let binding = Binding {
        mods: bind
            .mods
            .iter()
            .map(|m| match m {
                Mod::Alt => hyprland::bind_new::Mod::Alt,
                Mod::Ctrl => hyprland::bind_new::Mod::Ctrl,
                Mod::Super => hyprland::bind_new::Mod::Super,
                Mod::Shift => hyprland::bind_new::Mod::Shift,
            })
            .collect(),
        key: bind.key.to_string(),
        flags: flags_for_behavior(bind.behavior, desc),
        dispatcher,
    };
    trace!("binding exec: {binding:?}");
    binding.bind()?;
    Ok(())
}

fn flags_for_behavior(behavior: ExecBindBehavior, desc: String) -> Vec<Flag> {
    let mut flags = vec![Flag::AutoConsuming, Flag::Description(desc)];
    match behavior {
        ExecBindBehavior::PressOnce => {}
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
}

/// Remove any keybinds left by this application before registering the current set.
///
/// Hyprland keeps dynamically registered binds across config reloads, so merely
/// registering the current set can leave stale binds behind when the config changes.
#[tracing::instrument]
pub fn remove_hyprshell_keybinds() -> anyhow::Result<()> {
    let binds = Binds::get()?.to_vec();
    let hyprshell_binds = binds.into_iter().filter(|bind| {
        bind.description
            .starts_with(HYPRSHELL_BIND_DESCRIPTION_PREFIX)
    });

    for bind in hyprshell_binds {
        let mods = mods_from_mask(bind.modmask);
        let key = if bind.key.is_empty() {
            format!("code:{}", bind.keycode)
        } else {
            bind.key
        };

        tracing::trace!(?mods, %key, "unregistering hyprshell keybind");
        Binding {
            mods,
            key,
            // `bind_new::Binding::unbind` only uses the key and modifiers.
            dispatcher: Dispatch::ExecCmd(String::new(), None),
            flags: Vec::new(),
        }
        .unbind()?;
    }

    Ok(())
}

fn mods_from_mask(mask: u16) -> Vec<HyprlandMod> {
    // These are the XKB modifier-mask bits used by Hyprland's `binds` JSON.
    // Hyprshell keybinds use the four modifiers represented by its config.
    [
        (1, HyprlandMod::Shift),
        (4, HyprlandMod::Ctrl),
        (8, HyprlandMod::Alt),
        (64, HyprlandMod::Super),
    ]
    .into_iter()
    .filter_map(|(bit, modifier)| (mask & bit != 0).then_some(modifier))
    .collect()
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

    #[test]
    fn modifier_mask_is_converted_to_hyprland_modifiers() {
        assert_eq!(
            mods_from_mask(1 | 4 | 64),
            vec![HyprlandMod::Shift, HyprlandMod::Ctrl, HyprlandMod::Super]
        );
    }

    #[test]
    fn press_once_uses_normal_consuming_press_flags() {
        assert_eq!(
            flags_for_behavior(ExecBindBehavior::PressOnce, "overview".into()),
            vec![Flag::AutoConsuming, Flag::Description("overview".into())]
        );
    }

    #[test]
    fn release_binding_has_no_repeat_or_transparent_flags() {
        assert_eq!(
            flags_for_behavior(ExecBindBehavior::Release, "overview".into()),
            vec![
                Flag::AutoConsuming,
                Flag::Description("overview".into()),
                Flag::Release,
            ]
        );
    }

    #[test]
    fn switch_binding_flag_semantics_are_unchanged() {
        assert_eq!(
            flags_for_behavior(ExecBindBehavior::Repeating, "switch".into()),
            vec![
                Flag::AutoConsuming,
                Flag::Description("switch".into()),
                Flag::Repeating,
            ]
        );
        assert_eq!(
            flags_for_behavior(
                ExecBindBehavior::TransparentReleaseIgnoringMods {
                    unless_keys_down: None
                },
                "switch release".into()
            ),
            vec![
                Flag::AutoConsuming,
                Flag::Description("switch release".into()),
                Flag::Release,
                Flag::Transparent,
                Flag::IgnoreMods,
                Flag::NonConsuming,
            ]
        );
    }
}
