use crate::plugin::{LaunchItem, PluginReturn};
use crate::plugins::applications::data::{get_stored_runs, save_run};
use crate::plugins::applications::map::{DesktopAction, DesktopEntry, get_all_desktop_entries};
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use core_lib::util::analyse_exec;
use exec_lib::run::run_program;
use std::collections::HashMap;
use std::path::Path;
use tracing::{trace, warn};

impl LaunchItem {
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    fn from_desktop_entry(
        entry: &DesktopEntry,
        runs: &HashMap<Box<Path>, u64>,
        _show_execs: bool,
        _show_actions_submenu: bool,
    ) -> Self {
        // format as curve. formula: ln(x-2) / 0.025
        let runs = runs.get(&entry.source).unwrap_or(&0);
        let runs = ((*runs as f64) - 2.0).ln() / 0.025;

        Self {
            name: entry.name.clone(),
            keywords: entry.keywords.clone().into_boxed_slice(),
            icon: entry.icon.clone(),
            details: entry.exec_search.clone(),
            details_long: entry.type_search.clone(),
            bonus_score: runs.clamp(0.0, 60.0) as u64, // 30 runs equal 60
            iden: Identifier::data(
                PluginName::Applications,
                Box::from(entry.source.to_string_lossy()),
            ),
            enabled: true,
            takes_args: false,
            children: Box::from(
                entry
                    .actions
                    .iter()
                    .map(|i| Self::child_from_action(entry, i))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    fn child_from_action(entry: &DesktopEntry, action: &DesktopAction) -> Self {
        Self {
            name: action.name.clone(),
            keywords: entry.keywords.clone().into_boxed_slice(),
            icon: entry.icon.clone(),
            details: entry.exec_search.clone(),
            details_long: Some(action.exec.clone()),
            bonus_score: 0,
            iden: Identifier::data_additional(
                PluginName::Applications,
                Box::from(entry.source.to_string_lossy()),
                action.id.clone(),
            ),
            enabled: true,
            takes_args: false,
            children: Box::new([]),
        }
    }
}

pub fn get_launch_items(
    run_cache_weeks: u8,
    show_execs: bool,
    show_actions_submenu: bool,
    data_dir: &Path,
) -> Vec<LaunchItem> {
    let entries = get_all_desktop_entries();
    let runs = get_stored_runs(run_cache_weeks, data_dir);

    let mut matches = Vec::new();
    for entry in entries.iter() {
        matches.push(LaunchItem::from_desktop_entry(
            entry,
            &runs,
            show_execs,
            show_actions_submenu,
        ));
    }
    drop(entries);
    matches.sort_by(|a, b| b.bonus_score.cmp(&a.bonus_score));
    matches
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
                if let Some(action) = entry.actions.iter().find(|a| (*a.id).eq(&**section)) {
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
