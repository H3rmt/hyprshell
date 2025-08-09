use crate::{get_config_dirs, get_config_home, get_data_dirs};
use std::fs::DirEntry;
use tracing::{trace, warn};

pub fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    for dir in get_data_dirs() {
        if !dir.exists() {
            continue;
        }
        match dir.read_dir() {
            Ok(dir) => {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|e| e == "desktop") {
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

pub fn collect_mime_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    let mut dirs = Vec::new();
    // ensure correct order
    dirs.push(get_config_home());
    dirs.append(&mut get_config_dirs());
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
