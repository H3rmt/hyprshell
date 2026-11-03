use std::{env, path::PathBuf};

/// # Panics
/// if neither `XDG_DATA_HOME` nor HOME is set
///
/// returns `XDG_DATA_HOME` or `$HOME/.local/share`
#[must_use]
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
/// if neither `XDG_CACHE_HOME` nor HOME is set
///
/// Returns `XDG_CACHE_HOME` or `$HOME/.cache`
#[must_use]
pub fn get_cache_home() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.cache", home.to_string_lossy())))
        })
        .expect("Failed to get config dir (XDG_CACHE_HOME or HOME not set)")
}

/// # Panics
/// if neither `XDG_CONFIG_HOME` nor HOME is set
///
/// Returns `XDG_CONFIG_HOME` or `$HOME/.config`
#[must_use]
pub fn get_config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.config", home.to_string_lossy())))
        })
        .expect("Failed to get config dir (XDG_CONFIG_HOME or HOME not set)")
}

/// Returns `XDG_CONFIG_DIRS` or `/etc/xdg/`
#[must_use]
pub fn get_config_dirs() -> Vec<PathBuf> {
    env::var_os("XDG_CONFIG_DIRS").map_or_else(
        || vec![PathBuf::from("/etc/xdg/")],
        |val| env::split_paths(&val).collect(),
    )
}

/// Returns `XDG_DATA_DIRS` or `/usr/local/share` and `/usr/share`
#[must_use]
pub fn get_data_dirs() -> Vec<PathBuf> {
    env::var_os("XDG_DATA_DIRS").map_or_else(
        || {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        },
        |val| env::split_paths(&val).collect(),
    )
}
