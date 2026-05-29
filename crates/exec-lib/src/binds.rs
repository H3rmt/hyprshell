use core_lib::binds::ExecBind;
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

    let binding = Binding {
        mods: binds,
        key: bind.key.to_string(),
        flags: if bind.release {
            vec![
                Flag::Release,
                Flag::Transparent,
                Flag::AutoConsuming,
                Flag::Description(bind.desc.clone()),
            ]
        } else {
            vec![
                Flag::AutoConsuming,
                Flag::Repeating,
                Flag::Description(bind.desc.clone()),
            ]
        },
        dispatcher: Dispatch::ExecCmd(bind.exec.to_string(), None),
    };
    trace!("binding exec: {binding:?}");
    binding.unbind()?;
    binding.bind()?;
    Ok(())
}
