use crate::WarnWithDetails;
use anyhow::{Context, bail};
use notify::event::{DataChange, ModifyKind};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tracing::{debug, info, trace, warn};

pub fn hyprshell_config_listener<F>(
    file_path: &Path,
    callback: F,
) -> anyhow::Result<RecommendedWatcher>
where
    F: Fn(&'static str) + 'static + Clone + Send,
{
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist");
    }

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                callback("hyprshell config change");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err);
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .context("Failed to create watcher")?;

    info!("Starting hyprshell config reload listener");
    watcher
        .watch(file_path, RecursiveMode::NonRecursive)
        .context("Failed to start hyprshell config reload listener")?;

    Ok(watcher)
}

pub fn hyprshell_css_listener<F>(
    file_path: &Path,
    callback: F,
) -> anyhow::Result<RecommendedWatcher>
where
    F: Fn(&'static str) + 'static + Clone + Send,
{
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist");
    }

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                callback("hyprshell css change");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err);
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .context("Failed to create watcher")?;

    info!("Starting hyprshell css reload listener");
    watcher
        .watch(file_path.as_ref(), RecursiveMode::NonRecursive)
        .context("Failed to start hyprshell css reload listener")?;

    Ok(watcher)
}

pub fn hyprshell_config_block(file_path: &Path) -> anyhow::Result<()> {
    if !file_path.exists() {
        bail!("unable to watch for file changes as the file doesnt exist, exiting");
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                tx.send(()).warn_details("Failed to send reload signal");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err);
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .context("Failed to create watcher")?;
    debug!("Starting hyprshell config reload listener");

    watcher
        .watch(file_path.as_ref(), RecursiveMode::NonRecursive)
        .context("Failed to start hyprshell config reload listener")?;
    rx.recv().warn_details("Failed to receive reload signal");
    Ok(())
}
