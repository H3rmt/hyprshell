use anyhow::Context;
use core_lib::{Active, ClientId, Warn};
use hyprland::ctl::{Color, notify, reload};
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;
use hyprland::prelude::*;
use tracing::{Level, debug, span, trace, warn};

pub fn get_clients() -> Vec<Client> {
    Clients::get().map_or(vec![], |clients| clients.to_vec())
}

pub fn get_monitors() -> Vec<Monitor> {
    Monitors::get().map_or(vec![], |monitors| monitors.to_vec())
}

pub fn get_current_monitor() -> Option<Monitor> {
    Monitor::get_active().ok()
}

pub fn reload_config() {
    debug!("Reloading hyprland config");
    reload::call().warn("Failed to reload hyprland config");
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

pub fn activate_submap(submap_name: &str) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "submap").entered();
    Dispatch::call(DispatchType::Custom("submap", submap_name)).warn("unable to activate submap");
    debug!("Activated submap: {}", submap_name);
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
