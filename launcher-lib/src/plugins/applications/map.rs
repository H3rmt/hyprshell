use core_lib::{ExecType, IniFile, Warn, analyse_exec};
use std::fs::{DirEntry, read_to_string};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;
use tracing::{Level, span, trace, warn};

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub keywords: Vec<Box<str>>,
    pub exec_search: Box<str>,
    pub exec: Box<str>,
    pub exec_path: Option<Box<Path>>,
    pub type_search: &'static str,
    pub terminal: bool,
    pub source: Box<Path>,
    pub other: Vec<DesktopAction>,
}

#[derive(Debug, Clone)]
pub struct DesktopAction {
    pub id: Box<str>,
    pub name: Box<str>,
    pub exec: Box<str>,
}

fn get_desktop_file_map() -> &'static Mutex<Vec<DesktopEntry>> {
    static MAP_LOCK: OnceLock<Mutex<Vec<DesktopEntry>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn get_all_desktop_files<'a>() -> MutexGuard<'a, Vec<DesktopEntry>> {
    let map = get_desktop_file_map()
        .lock()
        .expect("Failed to lock desktop file map");
    map
}

pub fn reload_desktop_map(files: &[DirEntry]) {
    let mut map = get_desktop_file_map()
        .lock()
        .expect("Failed to lock desktop file map");
    map.clear();
    fill_desktop_file_map(&mut map, files).warn("Failed to fill desktop file map");
}

fn fill_desktop_file_map(map: &mut Vec<DesktopEntry>, files: &[DirEntry]) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "fill_desktop_file_map").entered();

    let now = Instant::now();
    for entry in files {
        if let Ok(str) = read_to_string(entry.path()) {
            let ini = IniFile::parse(&str);
            if let Some(section) = ini.get_section("Desktop Entry") {
                let r#type = section.get("Type");
                let no_display = section.get_boolean("NoDisplay");
                if r#type == Some("Application") && no_display.is_none_or(|n| !n) {
                    let name = section.get_boxed("Name");
                    let exec = section.get("Exec");
                    let icon = section.get_path_boxed("Icon");
                    let exec_path = section.get_path_boxed("Path");
                    let terminal = section.get_boolean("Terminal").unwrap_or(false);
                    let keywords = section
                        .get_boxed("Keywords")
                        .map(|k| k.split(';').map(|k| Box::from(k.trim())).collect())
                        .unwrap_or_else(Vec::new);

                    if let (Some(name), Some(exec)) = (name, exec) {
                        let mut exec = String::from(exec);
                        for replacement in ["%f", "%F", "%u", "%U"] {
                            exec = exec.replace(replacement, "");
                        }
                        let (exec_search, type_search) = match analyse_exec(&exec) {
                            ExecType::Flatpak(flatpak_identifier, _) => {
                                (flatpak_identifier, "flatpak")
                            }
                            ExecType::PWA(_, _) => (Box::from(""), "pwa"),
                            ExecType::FlatpakPWA(flatpak_identifier, _) => {
                                (flatpak_identifier, "flatpak-pwa")
                            }
                            ExecType::AppImage(app_image_identifier, _) => {
                                (app_image_identifier, "appimage")
                            }
                            ExecType::Absolute(exec_name, _) => (exec_name, ""),
                            ExecType::Relative(exec_name) => (exec_name, ""),
                        };

                        let other = ini
                            .sections()
                            .iter()
                            .filter_map(|(name, section)| {
                                if name.starts_with("Desktop Action ") {
                                    let exec = section.get_boxed("Exec")?;
                                    let name = section.get_boxed("Name")?;
                                    Some(DesktopAction {
                                        id: Box::from(name.trim_start_matches("Desktop Action ")),
                                        name,
                                        exec,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        map.push(DesktopEntry {
                            name,
                            icon,
                            keywords,
                            exec_search,
                            type_search,
                            exec_path,
                            terminal,
                            exec: exec.into_boxed_str(),
                            source: entry.path().into_boxed_path(),
                            other,
                        });
                    }
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
        "filled launcher desktop file map in {}ms",
        now.elapsed().as_millis()
    );
    Ok(())
}
