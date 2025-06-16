use anyhow::Context;
use core_lib::{Active, ClientId, Warn};
use hyprland::ctl::{Color, notify, reload};
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;
use hyprland::prelude::*;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, trace, warn};

pub fn get_clients() -> Vec<Client> {
    Clients::get().map_or(vec![], |clients| clients.to_vec())
}

pub fn get_monitors() -> Vec<Monitor> {
    Monitors::get().map_or(vec![], |monitors| monitors.to_vec())
}

pub fn get_current_monitor() -> Option<Monitor> {
    Monitor::get_active().ok()
}

pub fn reload_config() -> anyhow::Result<()> {
    debug!("Reloading hyprland config");
    reload::call().context("Failed to reload hyprland config")
}

pub fn toast(body: &str) {
    warn!("toast: {}", body);
    let _ = notify::call(
        notify::Icon::Warning,
        std::time::Duration::from_secs(10),
        Color::new(255, 0, 0, 255),
        format!("hyprshell Error: {}", body),
    );
}

pub fn apply_keybinds(list: Vec<(&str, String)>) {
    trace!("Applying binds and submaps");
    for (a, b) in list {
        trace!("{}={}", a, b);
        Keyword::set(a, b).warn("Failed to apply bind and submap");
    }
}

/// trim 0x from hexadecimal (base-16) string and convert to id
pub fn to_client_id(id: &hyprland::shared::Address) -> ClientId {
    u64::from_str_radix(id.to_string().trim_start_matches("0x"), 16)
        .expect("Failed to parse client id, this should never happen")
}
/// convert id to hexadecimal (base-16) string
pub fn to_client_address(id: ClientId) -> hyprland::shared::Address {
    hyprland::shared::Address::new(format!("{:x}", id))
}

fn get_prev_follow_mouse() -> &'static Mutex<Option<String>> {
    static PREV_FOLLOW_MOUSE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    PREV_FOLLOW_MOUSE.get_or_init(|| Mutex::new(None))
}

fn get_gestures_enabled() -> &'static Mutex<Option<bool>> {
    static GESTURES_ENABLED: OnceLock<Mutex<Option<bool>>> = OnceLock::new();
    GESTURES_ENABLED.get_or_init(|| Mutex::new(None))
}

pub fn set_remain_focused() -> anyhow::Result<()> {
    let follow = Keyword::get("input:follow_mouse").context("keyword failed")?;
    let mut lock = get_prev_follow_mouse()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_prev_follow_mouse mutex: {}", e))?;
    if follow.value.to_string() != "3" {
        trace!("Storing previous follow_mouse value: {}", follow.value);
        *lock = Some(follow.value.to_string());
    }
    Keyword::set("input:follow_mouse", "3").context("keyword failed")?;
    trace!("Set follow_mouse to 3");

    let gestures_enabled = Keyword::get("gestures:workspace_swipe").context("keyword failed")?;
    let mut lock = get_gestures_enabled()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_gestures_enabled mutex: {}", e))?;
    if gestures_enabled.set {
        trace!(
            "Storing previous gestures_enabled value: {}",
            gestures_enabled.value
        );
        *lock = Some(gestures_enabled.value.to_string() == "1");
    }
    Keyword::set("gestures:workspace_swipe", "0").context("keyword failed")?;
    trace!("Set gestures:workspace_swipe to 0");
    Ok(())
}

pub fn reset_remain_focused() -> anyhow::Result<()> {
    let follow = get_prev_follow_mouse()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_prev_follow_mouse mutex: {}", e))?;
    if let Some(follow) = follow.as_ref() {
        Keyword::set("input:follow_mouse", follow.to_string()).context("keyword failed")?;
        trace!("Restored previous follow_mouse value: {}", follow);
    } else {
        trace!("No previous follow_mouse value stored, skipping reset");
    }

    let gestures_enabled = get_gestures_enabled()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_gestures_enabled mutex: {}", e))?;
    if let Some(enabled) = gestures_enabled.as_ref() {
        Keyword::set("gestures:workspace_swipe", if *enabled { "1" } else { "0" })
            .context("keyword failed")?;
        trace!(
            "Restored previous gestures:workspace_swipe value: {}",
            enabled
        );
    } else {
        trace!("No previous gestures:workspace_swipe value stored, skipping reset");
    }
    Ok(())
}

pub fn activate_submap(submap_name: &str) -> anyhow::Result<()> {
    Dispatch::call(DispatchType::Custom("submap", submap_name)).context("dispatch failed")?;
    debug!("Activated submap: {}", submap_name);
    Ok(())
}

pub fn reset_submap() -> anyhow::Result<()> {
    Dispatch::call(DispatchType::Custom("submap", "reset")).context("dispatch failed")?;
    debug!("reset submap");
    Ok(())
}

pub fn get_initial_active() -> anyhow::Result<Active> {
    let active_client = Client::get_active()?.map(|c| to_client_id(&c.address));
    let active_ws = Workspace::get_active()?.id;
    let active_monitor = Monitor::get_active()?.id;
    Ok(Active {
        client: active_client,
        workspace: active_ws,
        monitor: active_monitor,
    })
}

pub fn get_version() -> anyhow::Result<String> {
    let version = hyprland::data::Version::get()
        .context("Failed to get version! (hyprland is probably outdated or too new??)")?;

    trace!("hyprland {version:?}");

    Ok(version
        .version
        .unwrap_or(version.tag.trim_start_matches('v').to_string()))
}
