use crate::plugin::{
    HighlightElement, HighlightedText, LaunchItem, MatchedLaunchItem, PluginReturn,
};
use core_lib::WarnWithDetails;
use core_lib::default::get_default_desktop_file;
use core_lib::transfer::{Identifier, PluginName};
use exec_lib::run::run_program;
use std::env;
use std::path::Path;
use tracing::{debug, trace, warn};

pub fn get_launch_items(text: &str) -> Vec<MatchedLaunchItem> {
    if text.starts_with('/') || text.starts_with('~') {
        // starting the file manager from bash works with ~,
        // checking if a file exists doesn't work with ~ as it is not expanded without a shell
        let text = if text.starts_with('~') {
            text.replacen('~', &env::var("HOME").unwrap_or_default(), 1)
        } else {
            text.to_string()
        };
        let exists = Path::new(&text).exists();
        let file_manager = get_file_manager_info();
        let item = LaunchItem {
            icon: file_manager.icon.clone(),
            name: format!("Open in {}", file_manager.name).into_boxed_str(),
            keywords: Box::from([]),
            details: Box::from(""),
            details_long: None,
            bonus_score: 0,
            takes_args: false,
            enabled: exists,
            iden: Identifier::plugin(PluginName::Path),
            children: Box::from([]),
        };
        vec![MatchedLaunchItem {
            highlight: HighlightElement::None,
            score: item.name.len() as u64,
            item,
        }]
    } else {
        vec![]
    }
}

pub fn launch_option(text: &str) -> PluginReturn {
    if text.is_empty() {
        debug!("No text to search for");
        return PluginReturn {
            show_animation: false,
        };
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
    run_program(&cmdline, None, false, None).warn_details("Failed to run program");
    PluginReturn {
        show_animation: true,
    }
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
            trace!("Found exec: {exec:?}, icon: {icon:?}");
            if let Some(exec) = exec {
                trace!(
                    "Found default file-manager file: {} with exec: {exec}",
                    entry.path().display()
                );
                return Some(Some(FilemanagerData { exec, name, icon }));
            }
        }
        None
    })
    .flatten()
    .unwrap_or_else(|| {
        warn!("No default browser found! (using firefox and gdbus to open)");
        FilemanagerData {
            exec: Box::from(r"nautilus --new-window %U"),
            icon: Some(Box::from(Path::new("org.gnome.Nautilus"))),
            name: Box::from(r"Nautilus"),
        }
    })
}
