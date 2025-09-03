use std::env;
use std::path::PathBuf;
use tracing::{trace, warn};

pub fn get_default_config_path() -> PathBuf {
    let mut path = get_config_home();
    #[cfg(debug_assertions)]
    path.push("hyprshell/config.debug.ron");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell/config.ron");

    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("toml");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    path.set_extension("json");
    if path.exists() {
        trace!("Found config file at {path:?}");
        return path;
    }

    #[cfg(feature = "json5_config")]
    {
        path.set_extension("json5");
        if path.exists() {
            trace!("Found config file at {path:?}");
            return path;
        }
    }

    path.set_extension("ron");
    path
}

pub fn get_default_css_path() -> PathBuf {
    let mut path = get_config_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell/styles.debug.css");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell/styles.css");
    path
}

pub fn get_default_data_dir() -> PathBuf {
    let mut path = get_data_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell.debug");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell");
    path
}

/// # Panics
/// if neither `XDG_DATA_HOME` nor HOME is set
pub fn get_data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.local/share", home.to_string_lossy())))
        })
        .expect("Failed to get config dir (XDG_DATA_HOME or HOME not set)")
}

/// # Panics
/// if neither `XDG_CONFIG_HOME` nor HOME is set
pub fn get_config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.config", home.to_string_lossy())))
        })
        .expect("Failed to get config dir (XDG_CONFIG_HOME or HOME not set)")
}

pub fn get_config_dirs() -> Vec<PathBuf> {
    env::var_os("XDG_CONFIG_DIRS").map_or_else(
        || vec![PathBuf::from("/etc/xdg/")],
        |val| env::split_paths(&val).collect(),
    )
}

pub fn get_data_dirs() -> Vec<PathBuf> {
    let mut dirs = env::var_os("XDG_DATA_DIRS").map_or_else(
        || {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        },
        |val| env::split_paths(&val).collect(),
    );

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from).map_or_else(
        || {
            env::var_os("HOME")
                .map(|p| PathBuf::from(p).join(".local/share"))
                .or_else(|| {
                    warn!("No XDG_DATA_HOME and HOME environment variable found");
                    None
                })
        },
        Some,
    ) {
        dirs.push(data_home);
    }

    dirs.into_iter()
        .map(|dir| dir.join("applications"))
        .collect()
}
