use std::path::PathBuf;
use tracing::trace;

use crate::env_dirs::{get_cache_home, get_config_home, get_data_dirs, get_data_home};

pub fn get_default_config_file() -> PathBuf {
    let mut path = get_config_home();
    #[cfg(debug_assertions)]
    path.push("hyprshell.debug/");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell/");

    path.push("config.toml");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("json");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("ron");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("json5");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("toml");
    path
}

#[must_use]
pub fn get_default_css_file() -> PathBuf {
    let mut path = get_config_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell.debug/styles.css");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell/styles.css");
    path
}

#[must_use]
pub fn get_default_data_dir() -> PathBuf {
    let mut path = get_data_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell.debug");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell");
    path
}

#[must_use]
pub fn get_default_cache_dir() -> PathBuf {
    let mut path = get_cache_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell.debug");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell");
    path
}

#[must_use]
pub fn get_system_data_dirs() -> Vec<PathBuf> {
    get_data_dirs()
        .into_iter()
        .filter_map(|mut path| {
            #[cfg(debug_assertions)]
            path.push("hyprshell.debug");
            #[cfg(not(debug_assertions))]
            path.push("hyprshell");
            if path.exists() { Some(path) } else { None }
        })
        .collect::<Vec<_>>()
}
