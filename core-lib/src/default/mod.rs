use crate::ini_owned::IniFileOwned;
use crate::{collect_desktop_files, collect_mime_files};
use anyhow::bail;
use std::collections::BTreeSet;
use std::fs::{DirEntry, read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock, RwLockReadGuard};
use std::{env, thread};
use tracing::{debug, debug_span, info_span, trace, warn};

fn get_desktop_files() -> &'static RwLock<Vec<(DirEntry, IniFileOwned)>> {
    static MAP_LOCK: OnceLock<RwLock<Vec<(DirEntry, IniFileOwned)>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(Default::default()))
}
fn get_mime_files() -> &'static RwLock<Vec<(DirEntry, IniFileOwned)>> {
    static MAP_LOCK: OnceLock<RwLock<Vec<(DirEntry, IniFileOwned)>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(Default::default()))
}
fn get_icons() -> &'static RwLock<BTreeSet<Box<str>>> {
    static MAP_LOCK: OnceLock<RwLock<BTreeSet<Box<str>>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(Default::default()))
}

pub fn get_all_desktop_files<'a>() -> RwLockReadGuard<'a, Vec<(DirEntry, IniFileOwned)>> {
    get_desktop_files()
        .read()
        .expect("Failed to lock desktop files mutex")
}
pub fn get_all_mime_files<'a>() -> RwLockReadGuard<'a, Vec<(DirEntry, IniFileOwned)>> {
    get_mime_files()
        .read()
        .expect("Failed to lock desktop files mutex")
}
pub fn get_all_icons<'a>() -> RwLockReadGuard<'a, BTreeSet<Box<str>>> {
    get_icons().read().expect("Failed to lock icon map")
}

pub fn theme_has_icon_name(name: &str) -> bool {
    let map = get_icons().read().expect("Failed to lock icon map");
    map.contains(&Box::from(name))
}

pub fn get_default_desktop_file<F, R>(mime: &str, r#fn: F) -> Option<R>
where
    F: FnOnce(&(DirEntry, IniFileOwned)) -> Option<R>,
{
    let mime_apps = get_mime_files().read().ok()?;
    let desktop_files = get_desktop_files().read().ok()?;

    for (_, ini) in mime_apps.iter() {
        if let Some(ini) = ini
            .get_section("Default Applications")
            .and_then(|section| section.get_first(mime))
            .or_else(|| {
                ini.get_section("Added Associations")
                    .and_then(|section| section.get_first(mime))
            })
            .and_then(|default| {
                desktop_files
                    .iter()
                    .find(|(entry, _)| entry.file_name() == *default)
            })
        {
            return r#fn(ini);
        }
    }
    None
}

/// Reloads desktop files and mime files from the system.
///
/// Stores them in global data mutexes.
pub fn reload_default_files() {
    let _span = tracing::span!(tracing::Level::TRACE, "reload_files").entered();
    let mut desktop_files_data = vec![];
    let mut mime_files_data = vec![];
    for file in collect_desktop_files() {
        let Ok(content) = read_to_string(file.path()) else {
            warn!("Failed to read desktop file: {}", file.path().display());
            continue;
        };
        let ini = IniFileOwned::from_str(&content);
        desktop_files_data.push((file, ini));
    }
    trace!("Collected all desktop files");

    for file in collect_mime_files() {
        let Ok(content) = read_to_string(file.path()) else {
            warn!("Failed to read desktop file: {}", file.path().display());
            continue;
        };
        let ini = IniFileOwned::from_str(&content);
        mime_files_data.push((file, ini));
    }
    trace!("Collected all mime files");

    let mut desktop_files = get_desktop_files()
        .write()
        .expect("Failed to lock global data mutex");
    *desktop_files = desktop_files_data;
    let mut mime_files = get_mime_files()
        .write()
        .expect("Failed to lock global data mutex");
    *mime_files = mime_files_data;
}

pub fn reload_available_icons(
    icon_names: Vec<String>,
    search_path: Vec<PathBuf>,
    in_background: bool,
) -> anyhow::Result<()> {
    let span = debug_span!("reload_icons");
    let _span = span.enter();

    let Ok(mut map) = get_icons().write() else {
        bail!("Failed to lock global data mutex");
    };
    debug!("found {} icons from theme", icon_names.len());
    map.clear();
    for icon in icon_names {
        map.insert(icon.into_boxed_str());
    }
    drop(map);

    if env::var_os("HYPRSHELL_NO_ALL_ICONS").is_none() {
        for path in search_path {
            let _span = span.clone();
            if path.exists() {
                if in_background {
                    thread::spawn(move || {
                        let _span = _span.entered();
                        let paths = collect_unique_filenames_recursive(&path);
                        debug!(
                            "found {} icons from filesystem in {path:?} paths (in background)",
                            paths.len()
                        );
                        let Ok(mut map) = get_icons().write() else {
                            warn!("Failed to lock global data mutex");
                            return;
                        };
                        map.extend(paths);
                        drop(map);
                    });
                } else {
                    let paths = collect_unique_filenames_recursive(&path);
                    debug!(
                        "found {} icons from filesystem in {path:?} paths",
                        paths.len()
                    );
                    let Ok(mut map) = get_icons().write() else {
                        bail!("Failed to lock global data mutex");
                    };
                    map.extend(paths);
                    drop(map);
                }
            }
        }
    }
    trace!("icon map filled");
    Ok(())
}

fn collect_unique_filenames_recursive(dir: &Path) -> BTreeSet<Box<str>> {
    let mut names = BTreeSet::new();
    let mut dirs_to_visit = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_visit.pop() {
        if current_dir.is_dir() {
            if let Ok(entries) = read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_visit.push(path);
                    } else if let Some(name_osstr) = path.file_stem() {
                        // Avoid allocation unless needed
                        let name = name_osstr.to_string_lossy();
                        if !name.is_empty() && !names.contains(&*name) {
                            names.insert(name.into_owned().into_boxed_str());
                        }
                    }
                }
            }
        }
    }
    names
}
