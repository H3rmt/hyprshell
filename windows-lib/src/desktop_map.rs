use core_lib::{IniFile, Warn};
use std::collections::HashMap;
use std::fs::{read_to_string, DirEntry};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tracing::{span, trace, warn, Level};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Source {
    DesktopFileName,
    DesktopFileStartupWmClass,
    DesktopFileExecName,
    ByPidExec,
}
type IconPathMap = HashMap<(Box<str>, Source), (Box<Path>, Box<Path>)>;

pub fn reload_desktop_map(files: &[DirEntry]) {
    let mut map = get_icon_path_map().lock().expect("Failed to lock icon map");
    map.clear();
    fill_desktop_file_map(&mut map, files).warn("Failed to fill desktop file map");
}

fn fill_desktop_file_map(map: &mut IconPathMap, files: &[DirEntry]) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "fill_desktop_file_map").entered();

    let now = Instant::now();
    for entry in files {
        if let Ok(str) = read_to_string(entry.path()) {
            let ini = IniFile::parse(&str);
            if let Some(section) = ini.get_section("Desktop Entry") {
                let name = section.get("Name");
                let icon = section.get("Icon");
                let startup_wm_class = section.get("StartupWMClass");
                let exec_name = section
                    .get("Exec")
                    .map(extract_exec_name)
                    .and_then(|l| l.split_whitespace().next())
                    .and_then(|l| l.split('/').next_back())
                    .map(|n| n.replace('"', ""));
                if let (Some(name), Some(icon)) = (name, icon) {
                    map.insert(
                        (Box::from(name.to_lowercase()), Source::DesktopFileName),
                        (Box::from(Path::new(icon)), entry.path().into_boxed_path()),
                    );
                }
                if let (Some(startup_wm_class), Some(icon)) = (startup_wm_class, icon) {
                    map.insert(
                        (
                            Box::from(startup_wm_class.to_lowercase()),
                            Source::DesktopFileStartupWmClass,
                        ),
                        (Box::from(Path::new(icon)), entry.path().into_boxed_path()),
                    );
                }
                if let (Some(exec_name), Some(icon)) = (exec_name, icon) {
                    map.insert(
                        (
                            Box::from(exec_name.to_lowercase()),
                            Source::DesktopFileExecName,
                        ),
                        (Box::from(Path::new(icon)), entry.path().into_boxed_path()),
                    );
                }
            } else {
                warn!(
                    "Failed to find section 'Desktop Entry' in file: {:?}",
                    entry.path()
                );
            }
        } else {
            warn!("Failed to read file: {:?}", entry.path());
        }
    }
    trace!(
        "filled icon desktop file map in {}ms",
        now.elapsed().as_millis()
    );
    Ok(())
}

fn extract_exec_name(l: &str) -> &str {
    // is a flatpak and isn't a PWA
    // (PWAs work out of the box by using the class being equal to the icon-name)
    // else chromium/chrome/etc would be detected as program icon which is not desired
    if l.contains("flatpak") && l.contains("--command") && !l.contains("--app-id") {
        // trim all text until --command
        l.split("--command=").last().unwrap_or(l)
    } else {
        l
    }
}

fn get_icon_path_map() -> &'static Mutex<IconPathMap> {
    static MAP_LOCK: OnceLock<Mutex<IconPathMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn add_path_for_icon_by_pid_exec(class: &str, path: Box<Path>) {
    let mut map = get_icon_path_map().lock().expect("Failed to lock icon map");
    map.insert(
        (Box::from(class.to_ascii_lowercase()), Source::ByPidExec),
        (path, Box::from(Path::new(""))),
    );
}

pub fn get_icon_name_by_name_from_desktop_files(
    name: &str,
) -> Option<(Box<Path>, Box<Path>, Source)> {
    let map = get_icon_path_map().lock().expect("Failed to lock icon map");
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
