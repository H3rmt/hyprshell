use crate::plugins::PluginReturn;
use crate::plugins::applications::data::{get_stored_runs, save_run};
use crate::plugins::applications::map::{DesktopEntry, get_all_desktop_entries};
use crate::plugins::LaunchItem;
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use core_lib::util::{ExecType, analyse_exec};
use exec_lib::run::run_program;
use std::collections::HashMap;
use std::path::Path;
use tracing::{trace, warn};

impl LaunchItem {
    fn from_desktop_entry(
        entry: &DesktopEntry,
        runs: &HashMap<Box<Path>, u64>,
        show_execs: bool,
        _show_actions_submenu: bool,
    ) -> Self {
        let (details, details_long) = if show_execs {
            match analyse_exec(&entry.exec) {
                ExecType::Flatpak(a, b) => (format!("[Flatpak] {a}").into_boxed_str(), Some(b)),
                ExecType::PWA(a, b) => (format!("[PWA] {a}").into_boxed_str(), Some(b)),
                ExecType::FlatpakPWA(a, b) => {
                    (format!("[Flatpak-PWA] {a}").into_boxed_str(), Some(b))
                }
                ExecType::AppImage(a, b) => (format!("[AppImage] {a}").into_boxed_str(), Some(b)),
                ExecType::Absolute(a, b) => (a, Some(b)),
                ExecType::Relative(a) => (a, None),
            }
        } else {
            (Box::from(""), None)
        };

        let runs = runs.get(&entry.source).unwrap_or(&0);
        Self {
            names: Box::from([entry.name.clone()]),
            keywords: entry.keywords.clone().into_boxed_slice(),
            icon: entry.icon.clone(),
            details,
            details_long,
            bonus_score: *runs,
            iden: Identifier::data(
                PluginName::Applications,
                Box::from(entry.source.to_string_lossy()),
            ),
            enabled: true,
            takes_args: false,
            children: Box::from([]),
        }
    }
}

pub fn get_launch_items(
    matches: &mut Vec<LaunchItem>,
    run_cache_weeks: u8,
    show_execs: bool,
    show_actions_submenu: bool,
    data_dir: &Path,
) {
    let entries = get_all_desktop_entries();
    let runs = get_stored_runs(run_cache_weeks, data_dir);

    for entry in entries.iter() {
        matches.push(LaunchItem::from_desktop_entry(
            entry,
            &runs,
            show_execs,
            show_actions_submenu,
        ));
    }
    drop(entries);
    // trace!("Added {count} applications to matches");
}
pub fn launch_option(
    data: Option<&str>,
    data_additional: Option<&str>,
    default_terminal: Option<&str>,
    data_dir: &Path,
) -> PluginReturn {
    let entries = get_all_desktop_entries();
    if let Some(data) = data {
        let entry = entries
            .iter()
            .find(|entry| data == entry.source.to_string_lossy());
        if let Some(entry) = entry {
            let exec = if let Some(section) = data_additional.as_ref() {
                // find desktop action
                if let Some(action) = entry.other.iter().find(|a| (*a.id).eq(&**section)) {
                    action.exec.clone()
                } else {
                    warn!(
                        "Failed to find action {:?} in entry {:?}",
                        &section, entry.name
                    );
                    return PluginReturn {
                        show_animation: false,
                    };
                }
            } else {
                entry.exec.clone()
            };
            run_program(
                &exec,
                entry.exec_path.as_deref(),
                entry.terminal,
                default_terminal,
            )
            .warn_details("Failed to run program");
            trace!("Saving run: {:?}", entry.source);
            save_run(&entry.source, data_dir).warn_details("Failed to cache run");
            return PluginReturn {
                show_animation: true,
            };
        }
        warn!("Failed to find entry for {data:?}|{data_additional:?}");
    }
    drop(entries);
    PluginReturn {
        show_animation: false,
    }
}
