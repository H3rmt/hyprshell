use anyhow::Context;
use core_lib::{Active, ClientId};
use hyprland::ctl::{Color, notify, reload};
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace};
use hyprland::keyword::Keyword;
use hyprland::prelude::*;
use semver::Version;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tracing::{debug, info, trace};

pub fn get_clients() -> Vec<Client> {
    Clients::get().map_or(vec![], hyprland::shared::HyprDataVec::to_vec)
}

pub fn get_monitors() -> Vec<Monitor> {
    Monitors::get().map_or(vec![], hyprland::shared::HyprDataVec::to_vec)
}

#[must_use]
pub fn get_current_monitor() -> Option<Monitor> {
    Monitor::get_active().ok()
}

pub fn reload_hyprland_config() -> anyhow::Result<()> {
    debug!("Reloading hyprland config");
    reload::call().context("Failed to reload hyprland config")
}

pub fn toast(body: &str) {
    let _ = notify::call(
        notify::Icon::Warning,
        Duration::from_secs(10),
        Color::new(255, 0, 0, 255),
        format!("hyprshell Error: {body}"),
    );
}

pub fn info_toast(body: &str, duration: Duration) {
    let _ = notify::call(
        notify::Icon::Info,
        duration,
        Color::new(0, 255, 0, 255),
        format!("hyprshell: {body}"),
    );
}

/// trim 0x from hexadecimal (base-16) string and convert to id
///
/// # Panics
/// Panics if the id cannot be parsed, this should never happen as the id is always a valid hexadecimal string
#[must_use]
pub fn to_client_id(id: &hyprland::shared::Address) -> ClientId {
    u64::from_str_radix(id.to_string().trim_start_matches("0x"), 16)
        .expect("Failed to parse client id, this should never happen")
}

/// convert id to hexadecimal (base-16) string
#[must_use]
pub fn to_client_address(id: ClientId) -> hyprland::shared::Address {
    hyprland::shared::Address::new(format!("{id:x}"))
}

fn get_prev_follow_mouse() -> &'static Mutex<Option<String>> {
    static PREV_FOLLOW_MOUSE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    PREV_FOLLOW_MOUSE.get_or_init(|| Mutex::new(None))
}

pub fn set_no_follow_mouse() -> anyhow::Result<()> {
    Keyword::set("input:follow_mouse", "3").context("keyword failed")?;
    trace!("Set follow_mouse to 3");
    Ok(())
}

pub fn reset_no_follow_mouse() -> anyhow::Result<()> {
    let follow = get_prev_follow_mouse()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_prev_follow_mouse mutex: {e:?}"))?;
    if let Some(follow) = follow.as_ref() {
        Keyword::set("input:follow_mouse", follow.clone()).context("keyword failed")?;
        trace!("Restored previous follow_mouse value: {follow}");
    } else {
        trace!("No previous follow_mouse value stored, skipping reset");
    }
    drop(follow);
    Ok(())
}

pub fn set_follow_mouse_default() -> anyhow::Result<()> {
    let mut lock = get_prev_follow_mouse()
        .lock()
        .map_err(|e| anyhow::anyhow!("unable to lock get_prev_follow_mouse mutex: {e:?}"))?;
    let follow = Keyword::get("input:follow_mouse").context("keyword failed")?;
    trace!("Storing previous follow_mouse value: {}", follow.value);
    *lock = Some(follow.value.to_string());
    drop(lock);
    Ok(())
}

pub fn get_initial_active() -> anyhow::Result<Active> {
    let active_client = Client::get_active()
        .ok()
        .flatten()
        .map(|c| to_client_id(&c.address));
    let active_ws = Workspace::get_active()
        .context("unable to get initial workspace")?
        .id;
    let active_monitor = Monitor::get_active()
        .context("unable to get initial monitor")?
        .id;
    Ok(Active {
        client: active_client,
        workspace: active_ws,
        monitor: active_monitor,
    })
}

pub fn check_version() -> anyhow::Result<()> {
    pub const MIN_VERSION: Version = Version::new(0, 42, 0);

    let version = hyprland::data::Version::get()
        .context("Failed to get version! (hyprland is probably outdated or too new??)")?;
    trace!("hyprland {version:?}");

    let version = version
        .version
        .unwrap_or_else(|| version.tag.trim_start_matches('v').to_string());
    info!(
        "Starting hyprshell {} in {} mode on hyprland {version}",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    );
    let parsed_version = Version::parse(&version).context("Unable to parse hyprland Version")?;
    if parsed_version.lt(&MIN_VERSION) {
        toast(&format!(
            "hyprland version {parsed_version} is too old or unknown, please update to at least {MIN_VERSION}",
        ));
    }
    Ok(())
}
