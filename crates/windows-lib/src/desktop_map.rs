use anyhow::Context;
use core_lib::default::get_all_desktop_files;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use tracing::{debug_span, trace, warn};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Source {
    DesktopFileName,
    DesktopFileStartupWmClass,
    DesktopFileExecName,
    ByPidExec,
}
type IconPathMap = HashMap<(Box<str>, Source), (Box<Path>, Box<Path>)>;
/// Display `Name` of the desktop file, keyed like [`IconPathMap`]
type AppNameMap = HashMap<(Box<str>, Source), Box<str>>;

fn get_icon_path_map() -> &'static RwLock<IconPathMap> {
    static MAP_LOCK: OnceLock<RwLock<IconPathMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_app_name_map() -> &'static RwLock<AppNameMap> {
    static MAP_LOCK: OnceLock<RwLock<AppNameMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn reload_class_to_icon_map() -> anyhow::Result<()> {
    let _span = debug_span!("reload_class_to_icon_map").entered();
    let mut map = get_icon_path_map()
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to lock icon path map"))?;
    let mut names = get_app_name_map()
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to lock app name map"))?;

    for (entry, ini) in get_all_desktop_files()
        .context("unable to get desktop files")?
        .iter()
    {
        if let Some(section) = ini.get_section("Desktop Entry") {
            let name = section.get_first("Name");
            let icon = section.get_first_as_path("Icon");
            let startup_wm_class = section.get_first("StartupWMClass");
            let exec_name = section.get_first("Exec").and_then(extract_exec_name);
            // names don't depend on the icon, so a missing icon doesn't hide the app name
            if let Some(name) = &name {
                names.insert(
                    (Box::from(name.to_lowercase()), Source::DesktopFileName),
                    Box::from(&**name),
                );
                if let Some(wm_class) = &startup_wm_class {
                    names.insert(
                        (
                            Box::from(wm_class.to_lowercase()),
                            Source::DesktopFileStartupWmClass,
                        ),
                        Box::from(&**name),
                    );
                }
                if let Some(exec_name) = &exec_name {
                    names.insert(
                        (
                            Box::from(exec_name.to_lowercase()),
                            Source::DesktopFileExecName,
                        ),
                        Box::from(&**name),
                    );
                }
            }
            if let (Some(name), Some(icon)) = (name, icon.clone()) {
                map.insert(
                    (Box::from(name.to_lowercase()), Source::DesktopFileName),
                    (icon, entry.path().into_boxed_path()),
                );
            }
            if let (Some(startup_wm_class), Some(icon)) = (startup_wm_class, icon.clone()) {
                map.insert(
                    (
                        Box::from(startup_wm_class.to_lowercase()),
                        Source::DesktopFileStartupWmClass,
                    ),
                    (icon, entry.path().into_boxed_path()),
                );
            }
            if let (Some(exec_name), Some(icon)) = (exec_name, icon) {
                map.insert(
                    (
                        Box::from(exec_name.to_lowercase()),
                        Source::DesktopFileExecName,
                    ),
                    (icon, entry.path().into_boxed_path()),
                );
            }
        } else {
            warn!(
                "Failed to find section 'Desktop Entry' in file: {}",
                entry.path().display()
            );
        }
    }
    drop(map);
    drop(names);
    trace!("filled class to icon map");
    Ok(())
}

fn extract_exec_name(line: Box<str>) -> Option<String> {
    // is a flatpak and isn't a PWA
    // (PWAs work out of the box by using the class being equal to the icon-name)
    // else chromium/chrome/etc would be detected as program icon which is not desired
    let exec =
        if line.contains("flatpak") && line.contains("--command") && !line.contains("--app-id") {
            // trim all text until --command
            line.split("--command=")
                .last()
                .map(Box::from)
                .unwrap_or(line)
        } else {
            line
        };
    exec.split_whitespace()
        .next()
        .and_then(|l| l.split('/').next_back())
        .map(|n| n.replace('"', ""))
}

pub fn add_path_for_icon_by_pid_exec(class: &str, path: Box<Path>) -> anyhow::Result<()> {
    let mut map = get_icon_path_map()
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to lock icon path map"))?;

    map.insert(
        (Box::from(class.to_ascii_lowercase()), Source::ByPidExec),
        (path, Box::from(Path::new(""))),
    );
    drop(map);
    Ok(())
}

pub fn get_icon_name_by_name_from_desktop_files(
    name: &str,
) -> Option<(Box<Path>, Box<Path>, Source)> {
    let Ok(map) = get_icon_path_map().read() else {
        warn!("Failed to lock icon path map");
        return None;
    };
    // prio: name by pid-exec, desktop file name, startup wm class, exec name
    map.get(&(Box::from(name.to_ascii_lowercase()), Source::ByPidExec))
        .map(|s| (s.0.clone(), s.1.clone(), Source::ByPidExec))
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileName,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileName))
        })
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileStartupWmClass,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileStartupWmClass))
        })
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileExecName,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileExecName))
        })
}

/// The `Name` of the desktop file that belongs to a window class, if there is one.
pub fn get_app_name_by_class(class: &str) -> Option<Box<str>> {
    let Ok(map) = get_app_name_map().read() else {
        warn!("Failed to lock app name map");
        return None;
    };
    let key: Box<str> = Box::from(class.to_ascii_lowercase());
    [
        Source::DesktopFileName,
        Source::DesktopFileStartupWmClass,
        Source::DesktopFileExecName,
    ]
    .into_iter()
    .find_map(|source| map.get(&(key.clone(), source)).cloned())
}
