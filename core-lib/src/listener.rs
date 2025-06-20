use crate::Warn;
use notify::event::{DataChange, ModifyKind};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tracing::{debug, info, trace, warn};

pub fn hyprshell_config_listener<F>(file_path: &Path, callback: F) -> Option<RecommendedWatcher>
where
    F: Fn(&'static str) + 'static + Clone + Send,
{
    if !file_path.exists() {
        debug!("unable to watch for file changes as the file doesnt exist");
        return None;
    }

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                callback("hyprshell config change");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err)
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .expect("Failed to create watcher");

    info!("Starting hyprshell config reload listener");
    watcher
        .watch(file_path, RecursiveMode::NonRecursive)
        .expect("Failed to start hyprshell config reload listener");

    Some(watcher)
}

pub fn hyprshell_css_listener<F>(file_path: &Path, callback: F) -> Option<RecommendedWatcher>
where
    F: Fn(&'static str) + 'static + Clone + Send,
{
    if !file_path.exists() {
        debug!("unable to watch for file changes as the file doesnt exist");
        return None;
    }

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                callback("hyprshell css change");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err)
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .expect("Failed to create watcher");

    info!("Starting hyprshell css reload listener");
    watcher
        .watch(file_path.as_ref(), RecursiveMode::NonRecursive)
        .expect("Failed to start hyprshell css reload listener");

    Some(watcher)
}

pub fn hyprshell_config_block(file_path: &Path) {
    if !file_path.exists() {
        debug!("unable to watch for file changes as the file doesnt exist, exiting");
        std::process::exit(1);
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| match res {
            Ok(event) if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                trace!("Event: {:?}", event);
                tx.send(()).warn("Failed to send reload signal");
            }
            Err(err) => {
                warn!("Watch error: {:?}", err)
            }
            Ok(_) => {}
        },
        Config::default(),
    )
    .expect("Failed to create watcher");
    debug!("Starting hyprshell config reload listener");

    watcher
        .watch(file_path.as_ref(), RecursiveMode::NonRecursive)
        .expect("Failed to start hyprshell config reload listener");
    rx.recv().warn("Failed to receive reload signal");
}
