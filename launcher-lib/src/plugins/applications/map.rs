use anyhow::Context;
use core_lib::default::get_all_desktop_files;
use core_lib::{ExecType, analyse_exec};
use std::path::Path;
use std::sync::{OnceLock, RwLock, RwLockReadGuard};
use tracing::{debug_span, trace, warn};

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

fn get_desktop_file_map() -> &'static RwLock<Vec<DesktopEntry>> {
    static MAP_LOCK: OnceLock<RwLock<Vec<DesktopEntry>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn get_all_desktop_entries<'a>() -> RwLockReadGuard<'a, Vec<DesktopEntry>> {
    get_desktop_file_map()
        .read()
        .expect("Failed to lock desktop files mutex")
}

pub fn reload_desktop_entries_map() -> anyhow::Result<()> {
    let _span = debug_span!("reload_desktop_entries_map").entered();

    let mut map = get_desktop_file_map()
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to lock desktop file map"))?;
    map.clear();
    for (entry, ini) in get_all_desktop_files()
        .context("unable to get all desktop files")?
        .iter()
    {
        if let Some(section) = ini.get_section("Desktop Entry") {
            let r#type = section.get_first("Type");
            let no_display = section.get_first_as_boolean("NoDisplay");
            if r#type.as_deref() == Some("Application") && no_display.is_none_or(|n| !n) {
                let name = section.get_first("Name");
                let exec = section.get_first("Exec");
                let icon = section.get_first_as_path("Icon");
                let exec_path = section.get_first_as_path("Path");
                let terminal = section.get_first_as_boolean("Terminal").unwrap_or(false);
                let keywords = section.get_all("Keywords").unwrap_or_else(Vec::new);

                if let (Some(name), Some(exec)) = (name, exec) {
                    let mut exec = String::from(exec);
                    for replacement in ["%f", "%F", "%u", "%U"] {
                        exec = exec.replace(replacement, "");
                    }
                    let (exec_search, type_search) = match analyse_exec(&exec) {
                        ExecType::Flatpak(flatpak_identifier, _) => (flatpak_identifier, "flatpak"),
                        ExecType::PWA(_, _) => (Box::from(""), "pwa"),
                        ExecType::FlatpakPWA(flatpak_identifier, _) => {
                            (flatpak_identifier, "flatpak-pwa")
                        }
                        ExecType::AppImage(app_image_identifier, _) => {
                            (app_image_identifier, "appimage")
                        }
                        ExecType::Absolute(exec_name, _) | ExecType::Relative(exec_name) => {
                            (exec_name, "")
                        }
                    };

                    let other = ini
                        .sections()
                        .iter()
                        .filter_map(|(name, section)| {
                            if name.starts_with("Desktop Action ") {
                                let exec = section.get_first("Exec")?;
                                let name_hr = section.get_first("Name")?;
                                Some(DesktopAction {
                                    id: name.clone(),
                                    name: name_hr,
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
    }
    drop(map);
    trace!("filled launcher desktop file map");
    Ok(())
}
