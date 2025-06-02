use core_lib::{find_config_dirs, get_config_dir, IniFile, Warn};
use std::fs::{read_to_string, DirEntry};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tracing::{debug, span, trace, warn, Level};

pub struct DefaultPlugins {
    pub exec: Box<str>,
    pub startup_wm_class: Option<Box<str>>,
    pub icon: Option<Box<Path>>,
}

pub(super) fn get_browser_info<'a>() -> MutexGuard<'a, DefaultPlugins> {
    BROWSER_EXEC
        .get()
        .expect("browser exec no initialized")
        .lock()
        .expect("Failed to lock browser exec")
}

static BROWSER_EXEC: OnceLock<Mutex<DefaultPlugins>> = OnceLock::new();

pub fn reload_default_browser(files: &[DirEntry]) {
    let _span = span!(Level::TRACE, "reload_default_browser").entered();
    let default_browser = get_default_browser_desktop_file();

    for entry in files {
        if entry.file_name() == default_browser.as_deref().unwrap_or_default() {
            if let Ok(str) = read_to_string(entry.path()) {
                let ini = IniFile::parse(&str);
                if let Some(section) = ini.get_section("Desktop Entry") {
                    let exec = section.get("Exec");
                    let startup_wm_class = section.get("StartupWMClass");
                    let icon = section.get("Icon");
                    trace!(
                        "Found exec: {:?}, startup_wm_class: {:?}, icon: {:?}",
                        exec,
                        startup_wm_class,
                        icon
                    );
                    if let Some(exec) = exec {
                        trace!("Found default browser file: {:?} with exec: {:?} and startup_wm_class: {:?}", entry.path(), exec, startup_wm_class);
                        let _ = BROWSER_EXEC.set(Mutex::new(DefaultPlugins {
                            exec: Box::from(exec),
                            startup_wm_class: startup_wm_class.map(Box::from),
                            icon: icon.map(Path::new).map(Box::from),
                        }));
                        return;
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
    }
    warn!("No default browser found! (using firefox and gdbus to open)");
    let _ = BROWSER_EXEC.set(Mutex::new(DefaultPlugins {
        exec: Box::from(r#"gdbus call --session --dest="org.freedesktop.portal.Desktop" --object-path=/org/freedesktop/portal/desktop --method=org.freedesktop.portal.OpenURI.OpenURI '' '%u' '{}'"#),
        startup_wm_class: Some(Box::from("firefox")),
        icon: Some(Box::from(Path::new("firefox"))),
    }));
}

fn get_default_browser_desktop_file() -> Option<Box<str>> {
    for entry in get_mimeapps() {
        // parse the mimeapps.list file
        if let Ok(str) = read_to_string(entry.path()) {
            let ini = IniFile::parse(&str);
            let browser = ini
                .get_section("Default Applications")?
                .get_boxed("x-scheme-handler/https")?;
            debug!("Default browser from mimeapps.list: {}", browser);
            return Some(browser);
        } else {
            warn!("Failed to read file: {:?}", entry.path());
        }
    }
    None
}

fn get_mimeapps() -> Vec<DirEntry> {
    let mut res = Vec::new();
    let mut dirs = find_config_dirs();
    dirs.push(get_config_dir());
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
                continue;
            }
        }
    }
    debug!("found {} mimeapps lists", res.len());
    res
}
