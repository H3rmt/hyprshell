use anyhow::Context;
use core_lib::binds::ExecBind;
use core_lib::{LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE, SWITCH_NAMESPACE};
use hyprland::config::binds;
use hyprland::config::binds::{Binder, Binding, Flag};
use hyprland::default_instance_panic;
use hyprland::dispatch::DispatchType;
use hyprland::keyword::Keyword;
use tracing::{trace, warn};

pub fn apply_layerrules() -> anyhow::Result<()> {
    let instance = default_instance_panic();
    Keyword::set(
        instance,
        "layerrule",
        format!("noanim, {LAUNCHER_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("ignorezero, {LAUNCHER_NAMESPACE}"),
    )?;
    Keyword::set(instance, "layerrule", format!("blur, {LAUNCHER_NAMESPACE}"))?;
    Keyword::set(
        instance,
        "layerrule",
        format!("xray 0, {LAUNCHER_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("blurpopups, {LAUNCHER_NAMESPACE}"),
    )?;

    Keyword::set(
        instance,
        "layerrule",
        format!("noanim, {OVERVIEW_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("dimaround, {OVERVIEW_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("ignorezero, {OVERVIEW_NAMESPACE}"),
    )?;
    Keyword::set(instance, "layerrule", format!("blur, {OVERVIEW_NAMESPACE}"))?;
    Keyword::set(
        instance,
        "layerrule",
        format!("xray 0, {OVERVIEW_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("blurpopups, {OVERVIEW_NAMESPACE}"),
    )?;

    Keyword::set(instance, "layerrule", format!("noanim, {SWITCH_NAMESPACE}"))?;
    Keyword::set(
        instance,
        "layerrule",
        format!("dimaround, {SWITCH_NAMESPACE}"),
    )?;
    Keyword::set(
        instance,
        "layerrule",
        format!("ignorezero, {SWITCH_NAMESPACE}"),
    )?;
    Keyword::set(instance, "layerrule", format!("blur, {SWITCH_NAMESPACE}"))?;
    Keyword::set(instance, "layerrule", format!("xray 0, {SWITCH_NAMESPACE}"))?;
    Keyword::set(
        instance,
        "layerrule",
        format!("blurpopups, {SWITCH_NAMESPACE}"),
    )?;
    Ok(())
}

// ctrl+shift+alt, h
// hyprland::bind!(d e | SUPER, Key, "a" => Exec, "pkill hyprshell");
pub fn apply_exec_bind(bind: &ExecBind) -> anyhow::Result<()> {
    let instance = default_instance_panic();
    let binding = Binding {
        mods: bind
            .mods
            .iter()
            .flat_map(|m| match m.to_lowercase().as_str() {
                "alt" => Some(binds::Mod::ALT),
                "control" | "ctrl" => Some(binds::Mod::CTRL),
                "super" | "win" => Some(binds::Mod::SUPER),
                "shift" => Some(binds::Mod::SHIFT),
                _ => {
                    warn!("unknown mod: {}", m);
                    None
                }
            })
            .collect(),
        key: binds::Key::Key(&bind.key),
        flags: if bind.on_release {
            vec![Flag::r, Flag::t]
        } else {
            vec![]
        },
        dispatcher: DispatchType::Exec(&bind.exec),
    };
    trace!("binding exec: {binding:?}");
    Binder::bind(instance, binding).with_context(|| format!("binding exec failed: {bind:?}"))?;
    Ok(())
}
