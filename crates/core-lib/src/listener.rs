use anyhow::{Context, bail};
use inotify::{Inotify, WatchMask};
use std::path::Path;
use tracing::{info, trace};

pub fn hyprshell_config_listener(file_path: &Path) -> anyhow::Result<Inotify> {
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist");
    }
    let inotify = Inotify::init().context("Failed to create watcher")?;
    info!("Starting hyprshell config reload listener");

    inotify
        .watches()
        .add(file_path, WatchMask::MODIFY)
        .context("Failed to start hyprshell config reload listener")?;

    Ok(inotify)
}

pub fn hyprshell_css_listener(file_path: &Path) -> anyhow::Result<Inotify> {
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist");
    }
    let inotify = Inotify::init().context("Failed to create watcher")?;
    info!("Starting hyprshell css reload listener");

    inotify
        .watches()
        .add(file_path, WatchMask::MODIFY)
        .context("Failed to start hyprshell css reload listener")?;

    Ok(inotify)
}

pub fn hyprshell_config_block(file_path: &Path) -> anyhow::Result<()> {
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist, exiting");
    }
    let mut inotify = Inotify::init().context("Failed to create watcher")?;
    info!("Starting hyprshell config reload listener");

    inotify
        .watches()
        .add(file_path, WatchMask::MODIFY)
        .context("Failed to start hyprshell config reload listener")?;

    let mut buffer = [0u8; 4096];
    loop {
        match inotify.read_events_blocking(&mut buffer) {
            Ok(events) => {
                trace!("Received events: {events:?}");
                for event in events {
                    if event.mask.contains(inotify::EventMask::MODIFY) {
                        trace!("Event: {event:?}");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                bail!("Failed to read events: {e}");
            }
        }
    }
}
