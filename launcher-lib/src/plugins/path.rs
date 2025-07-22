use crate::plugins::{Identifier, PluginNames, SortableLaunchOption};
use core_lib::WarnWithDetails;
use core_lib::default::get_default_desktop_file;
use exec_lib::run::run_program;
use std::path::Path;
use tracing::{debug, trace, warn};

pub fn get_path_options(matches: &mut Vec<SortableLaunchOption>, text: &str) {
    if text.starts_with("/") || text.starts_with("~") {
        let file_manager = get_file_manager_info();
        matches.push(SortableLaunchOption {
            icon: file_manager.icon.clone(),
            name: format!("Open in {}", file_manager.name).into_boxed_str(),
            details: Box::from(""),
            details_long: None,
            score: 100,
            iden: Identifier::plugin(PluginNames::Path),
            details_menu: vec![],
        });
    }
}

pub fn launch_option(text: &str) -> bool {
    if text.is_empty() {
        debug!("No text to search for");
        return false;
    }

    debug!("Opening folder: {}", text);
    let file_manager = get_file_manager_info();
    let cmdline = if ["%u", "%U", "%f", "%F"]
        .iter()
        .any(|repl| file_manager.exec.contains(repl))
    {
        let mut exec = file_manager.exec.to_string();
        for repl in ["%u", "%U", "%f", "%F"] {
            exec = exec.replace(repl, text);
        }
        exec
    } else {
        format!("{} {}", file_manager.exec, text)
    };
    debug!("Launching file-manger: {}", cmdline);
    run_program(&cmdline, None, false, &None).warn_details("Failed to run program");
    true
}

pub struct FilemanagerData {
    pub exec: Box<str>,
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
}

pub(super) fn get_file_manager_info() -> FilemanagerData {
    get_default_desktop_file("inode/directory", |(entry, ini)| {
        if let Some(section) = ini.get_section("Desktop Entry") {
            let exec = section.get_first("Exec");
            let icon = section.get_first_as_path("Icon");
            let name = section.get_first("Name").unwrap_or_default();
            trace!("Found exec: {:?}, icon: {:?}", exec, icon);
            if let Some(exec) = exec {
                trace!(
                    "Found default file-manager file: {:?} with exec: {:?}",
                    entry.path(),
                    exec,
                );
                return Some(Some(FilemanagerData { exec, icon, name }));
            }
        }
        None
    })
    .flatten()
    .unwrap_or_else(|| {
        warn!("No default browser found! (using firefox and gdbus to open)");
        FilemanagerData {
            exec: Box::from(r#"nautilus --new-window %U"#),
            icon: Some(Box::from(Path::new("org.gnome.Nautilus"))),
            name: Box::from(r#"Nautilus"#),
        }
    })
}
