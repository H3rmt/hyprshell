use anyhow::{Context, anyhow};
use core_lib::{Active, ClientId, notify_warn};
use hyprland::ctl::reload;
use hyprland::data::{Client, Monitor, Workspace};
use hyprland::keyword::Keyword;
use hyprland::prelude::*;
use semver::Version;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, trace, warn};

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

pub fn reload_hyprland_config() -> anyhow::Result<()> {
    debug!("Reloading hyprland config");
    reload::call().context("Failed to reload hyprland config")
}

/// tries to get initial data for 500 ms * 40 = 20 s
///
/// # Errors
/// Returns an error if the initial data is not available after 5000 ms
pub fn get_initial_active() -> anyhow::Result<Active> {
    let mut tries = 0;
    loop {
        match internal_get_initial_active() {
            Ok(a) => break Ok(a),
            Err(e) => {
                if tries > 40 {
                    break Err(e);
                }
                warn!("waiting for correct initial active state from hyprland: {e:?}");
                thread::sleep(Duration::from_millis(500));
            }
        }
        tries += 1;
    }
}

fn internal_get_initial_active() -> anyhow::Result<Active> {
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
    pub const MIN_VERSION: Version = Version::new(0, 52, 1);

    let version = get_version()
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
        notify_warn(&format!(
            "hyprland version {parsed_version} is too old or unknown, please update to at least {MIN_VERSION}",
        ));
    }
    Ok(())
}

pub(crate) fn get_version() -> anyhow::Result<hyprland::data::Version> {
    let mut tries = 0;
    loop {
        match hyprland::data::Version::get() {
            Ok(a) => break Ok(a),
            Err(e) => {
                if tries > 40 {
                    break Err(anyhow!(e));
                }
                warn!("waiting for correct version from hyprland: {e:?}");
                thread::sleep(Duration::from_millis(500));
            }
        }
        tries += 1;
    }
}
