use anyhow::Context;
use core_lib::binds::ExecBind;
use core_lib::{LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE, SWITCH_NAMESPACE};
use hyprland::bind_new::{Binding, Flag, Mod};
use hyprland::config::binds;
use hyprland::dispatch::DispatchType;
use hyprland::dispatch_new::Dispatch;
use hyprland::keyword::Keyword;
use hyprland::window_rule::{LayerEffect, LayerMatch, LayerRule};
use tracing::{trace, warn};

pub fn apply_layerrules() -> anyhow::Result<()> {
    if let Err(e) = apply_layerrules_lua() {
        warn!("Failed to apply layerrules: {}, trying legacy syntax", e);
        return apply_layerrules_legacy();
    }
    Ok(())
}

pub fn apply_layerrules_lua() -> anyhow::Result<()> {
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

pub fn apply_layerrules_legacy() -> anyhow::Result<()> {
    // TODO add option to enable blur
    Keyword::set("layerrule", format!("noanim, {LAUNCHER_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("xray 0, {LAUNCHER_NAMESPACE}"))?;

    Keyword::set("layerrule", format!("noanim, {OVERVIEW_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("xray 0, {OVERVIEW_NAMESPACE}"))?;

    Keyword::set("layerrule", format!("noanim, {SWITCH_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("dimaround, {SWITCH_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("xray 0, {SWITCH_NAMESPACE}"))?;
    Ok(())
}

pub fn apply_exec_bind(bind: &ExecBind) -> anyhow::Result<()> {
    if let Err(e) = apply_exec_bind_lua(bind) {
        warn!("Failed to apply keybinds: {}, trying legacy syntax", e);
        return apply_exec_bind_legacy(bind);
    }
    Ok(())
}

pub fn apply_exec_bind_lua(bind: &ExecBind) -> anyhow::Result<()> {
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
        dispatcher: Dispatch::ExecCmd(bind.exec.clone(), None),
    };
    trace!("binding exec: {binding:?}");
    binding.unbind()?;
    binding.bind()?;
    Ok(())
}

pub fn apply_exec_bind_legacy(bind: &ExecBind) -> anyhow::Result<()> {
    let mods = bind
        .mods
        .iter()
        .filter_map(|m| match m.to_lowercase().as_str() {
            "alt" => Some(binds::Mod::ALT),
            "control" | "ctrl" => Some(binds::Mod::CTRL),
            "super" | "win" => Some(binds::Mod::SUPER),
            "shift" => Some(binds::Mod::SHIFT),
            _ => {
                warn!("unknown mod: {m}");
                None
            }
        })
        .collect::<Vec<_>>();
    let binding = binds::Binding {
        mods: mods.as_slice(),
        key: binds::Key::Key(&bind.key),
        flags: if bind.release {
            &vec![binds::Flag::r, binds::Flag::t]
        } else {
            &vec![binds::Flag::e]
        },
        dispatcher: DispatchType::Exec(&bind.exec),
    };
    trace!("binding exec: {binding:?}");
    binds::Binder::bind(binding).with_context(|| format!("binding exec failed: {bind:?}"))?;
    Ok(())
}
