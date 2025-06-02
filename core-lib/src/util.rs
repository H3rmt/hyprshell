use crate::find_application_dirs;
use anyhow::Context;
use semver::Version;
use std::fs::DirEntry;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::{env, fmt};
use tracing::{debug, info, warn};

pub const MIN_VERSION: Version = Version::new(0, 42, 0);

pub const OVERVIEW_NAMESPACE: &str = "hyprshell_overview";
pub const LAUNCHER_NAMESPACE: &str = "hyprshell_launcher";

pub trait Warn<A> {
    fn warn(self, msg: &str) -> Option<A>;
}

impl<A> Warn<A> for Option<A> {
    fn warn(self, msg: &str) -> Option<A> {
        match self {
            Some(o) => Some(o),
            None => {
                warn!("{}", msg);
                None
            }
        }
    }
}

impl<A, E: fmt::Debug + fmt::Display> Warn<A> for Result<A, E> {
    fn warn(self, msg: &str) -> Option<A> {
        match self {
            Ok(o) => Some(o),
            Err(e) => {
                warn!("{}: {}", msg, e);
                debug!("{e:?}");
                None
            }
        }
    }
}

// from https://github.com/i3/i3/blob/next/i3-sensible-terminal
// shorted to only the most common ones that I know support -e option
pub const TERMINALS: [&str; 9] = [
    "alacritty",
    "kitty",
    "wezterm",
    "foot",
    "qterminal",
    "lilyterm",
    "tilix",
    "terminix",
    "konsole",
];

pub fn get_daemon_socket_path_buff() -> PathBuf {
    let mut buf = if let Some(runtime_path) = env::var_os("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(runtime_path)
    } else if let Ok(uid) = env::var("UID") {
        std::path::PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        std::path::PathBuf::from("/tmp")
    };
    #[cfg(debug_assertions)]
    buf.push("hyprshell.debug.sock");
    #[cfg(not(debug_assertions))]
    buf.push("hyprshell.sock");
    buf
}

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_daemon_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf).is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

pub fn check_version(version: anyhow::Result<String>) -> anyhow::Result<()> {
    if let Ok(version) = version {
        info!(
            "Starting hyprshell {} in {} mode on hyprland {}",
            env!("CARGO_PKG_VERSION"),
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            version,
        );
        let parsed_version =
            Version::parse(&version).context("Unable to parse hyprland Version")?;
        if parsed_version.lt(&MIN_VERSION) {
            Err(anyhow::anyhow!(
                "hyprland version {} is too old or unknown, please update to at least {}",
                parsed_version,
                MIN_VERSION
            ))
        } else {
            Ok(())
        }
    } else {
        Err(anyhow::anyhow!("Unable to get hyprland version"))
    }
}

pub fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    for dir in find_application_dirs() {
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
                continue;
            }
        }
    }
    debug!("found {} desktop files", res.len());
    res
}

pub fn get_hyprshell_path() -> String {
    env::current_exe()
        .expect("Current executable not found")
        .display()
        .to_string()
        .replace("(deleted)", "")
}

pub fn get_hyprctl_path() -> String {
    env::var("PATH")
        .unwrap_or_else(|_| String::from("/usr/bin:/bin:/usr/local/bin"))
        .split(':')
        .find_map(|dir| {
            let path = PathBuf::from(dir).join("hyprctl");
            if path.exists() {
                Some(path.display().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| String::from("hyprctl"))
}

pub fn generate_socat(echo: &str) -> String {
    format!(r#"{} socat '{}'"#, get_hyprshell_path(), echo)
}

pub fn generate_socat_and_activate_submap(echo: &str, submap: &str) -> String {
    format!(
        r#"{} dispatch submap {} && {} socat '{}'"#,
        get_hyprctl_path(),
        submap,
        get_hyprshell_path(),
        echo
    )
}

#[derive(Debug, Clone)]
pub enum ExecType {
    Flatpak(Box<str>, Box<str>),
    PWA(Box<str>, Box<str>),
    FlatpakPWA(Box<str>, Box<str>),
    Absolute(Box<str>, Box<str>),
    AppImage(Box<str>, Box<str>),
    Relative(Box<str>),
}

const UNKNOWN_EXEC: &str = "unknown";

pub fn analyse_exec(exec: &str) -> ExecType {
    let exec_trim = exec.replace("'", "").replace("\"", "");
    // pwa detection
    if exec.contains("--app-id=") && exec.contains("--profile-directory=") {
        // "flatpak 'run'" = pwa from browser inside flatpak
        if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
            let browser_exec_in_flatpak = exec_trim
                .split_whitespace()
                .find(|s| s.contains("--command="))
                .and_then(|s| {
                    s.split('=')
                        .next_back()
                        .and_then(|s| s.split('/').next_back())
                })
                .unwrap_or(UNKNOWN_EXEC);
            let flatpak_identifier = exec_trim
                .split_whitespace()
                .skip(2)
                .find(|arg| !arg.starts_with("--"))
                .unwrap_or(UNKNOWN_EXEC);
            ExecType::FlatpakPWA(
                Box::from(flatpak_identifier),
                Box::from(browser_exec_in_flatpak),
            )
        } else {
            // normal PWA
            let browser_exec = exec
                .split_whitespace()
                .next()
                .and_then(|s| s.split('/').next_back())
                .unwrap_or(UNKNOWN_EXEC);
            let browser_full_exec = exec.split_whitespace().next().unwrap_or(UNKNOWN_EXEC);
            ExecType::PWA(Box::from(browser_exec), Box::from(browser_full_exec))
        }
        // flatpak detection
    } else if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
        let command_in_flatpak = exec_trim
            .split_whitespace()
            .find(|s| s.contains("--command="))
            .and_then(|s| {
                s.split('=')
                    .next_back()
                    .and_then(|s| s.split('/').next_back())
            })
            .unwrap_or(UNKNOWN_EXEC);
        let flatpak_identifier = exec_trim
            .split_whitespace()
            .skip(2)
            .find(|arg| !arg.starts_with("--"))
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::Flatpak(Box::from(flatpak_identifier), Box::from(command_in_flatpak))
    } else if exec_trim.contains(".AppImage") {
        // AppImage detection
        let appimage_name = exec_trim
            .split_whitespace()
            .next()
            .and_then(|s| s.split('/').next_back())
            .and_then(|s| s.split('_').next())
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::AppImage(Box::from(appimage_name), Box::from(exec))
    } else if exec_trim.starts_with("/") {
        let exec_name = exec_trim
            .split_whitespace()
            .next()
            .and_then(|s| s.split('/').next_back())
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::Absolute(Box::from(exec_name), Box::from(exec))
    } else {
        ExecType::Relative(Box::from(exec_trim))
    }
}
