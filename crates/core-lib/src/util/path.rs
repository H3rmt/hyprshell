use crate::env_dirs::{get_config_dirs, get_config_home, get_data_dirs, get_data_home};
use std::fs::DirEntry;
use tracing::{trace, warn};

/// Collects all .desktop files from standard directories
/// according to the XDG Base Directory Specification.
pub fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    let dirs = {
        // ensure correct order
        let mut dirs = Vec::new();
        dirs.push(get_data_home().join("applications"));
        get_data_dirs()
            .iter()
            .map(|d| d.join("applications"))
            .for_each(|d| dirs.push(d));
        dirs
    };
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        match dir.read_dir() {
            Ok(dir) => {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().is_some_and(|e| e == "desktop")
                        && !res
                            .iter()
                            .any(|e: &DirEntry| e.file_name() == entry.file_name())
                    {
                        res.push(entry);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read dir {dir:?}: {e}");
            }
        }
    }
    trace!("found {} desktop files", res.len());
    res
}

/// Collects all mimeapps.list files from standard directories
/// according to the XDG Base Directory Specification.
pub fn collect_mime_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    let dirs = {
        // ensure correct order
        let mut dirs = Vec::new();
        dirs.push(get_config_home());
        dirs.append(&mut get_config_dirs());
        dirs.push(get_data_home().join("applications"));
        get_data_dirs()
            .iter()
            .map(|d| d.join("applications"))
            .for_each(|d| dirs.push(d));
        dirs
    };
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        match dir.read_dir() {
            Ok(dir) => {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path
                            .file_name()
                            .is_some_and(|e| e.to_string_lossy().ends_with("mimeapps.list"))
                    {
                        res.push(entry);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read dir {dir:?}: {e}");
            }
        }
    }
    trace!("found {} mimeapps lists", res.len());
    res
}
