use std::env;
use std::path::PathBuf;
use tracing::trace;

pub fn get_default_config_file() -> PathBuf {
    let mut path = get_config_home();
    #[cfg(debug_assertions)]
    path.push("hyprshell.debug/");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell/");
    path.push("config.ron");
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
pub fn get_default_system_data_dir() -> PathBuf {
    let mut path = get_system_data_home();

    #[cfg(debug_assertions)]
    path.push("hyprshell.debug");
    #[cfg(not(debug_assertions))]
    path.push("hyprshell");
    path
}

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

/// returns `/usr/share`
#[must_use]
pub fn get_system_data_home() -> PathBuf {
    PathBuf::from("/usr/share")
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
