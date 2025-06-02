use crate::plugins::applications::data::{get_stored_runs, save_run};
use crate::plugins::applications::map::{get_all_desktop_files, DesktopEntry};
use crate::plugins::{DetailsMenuItem, Identifier, PluginNames, SortableLaunchOption};
use core_lib::{analyse_exec, ExecType, Warn};
use exec_lib::run::run_program;
use std::collections::HashMap;
use std::path::Path;
use tracing::{trace, warn};

#[derive(Debug, Clone, Copy)]
enum MatchType {
    AppType = 1,
    Keyword = 4,
    ExecName = 10,
    ExecExact = 15,
    Name = 16,
    Exact = 21,
}

impl SortableLaunchOption {
    fn from_desktop_entry(
        entry: &DesktopEntry,
        r#match: MatchType,
        runs: &HashMap<Box<Path>, u64>,
        show_execs: bool,
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
        // trace!(
        //     "entry {} has match:{:?}({}) and runs:{} => {}",
        //     entry.name,
        //     r#match,
        //     r#match as u64,
        //     runs,
        //     r#match as u64 + runs
        // );
        SortableLaunchOption {
            name: entry.name.clone(),
            icon: entry.icon.clone(),
            details,
            details_long,
            score: r#match as u64 + runs,
            iden: Identifier {
                identifier: Some(Box::from(entry.source.to_string_lossy())),
                plugin: PluginNames::Applications,
            },
            details_menu: entry
                .other
                .iter()
                .map(|action| DetailsMenuItem {
                    text: action.name.clone(),
                    exec: action.exec.clone(),
                    iden: Identifier {
                        identifier: Some(Box::from(format!(
                            "{}|{}",
                            entry.source.to_string_lossy(),
                            action.id
                        ))),
                        plugin: PluginNames::Applications,
                    },
                })
                .collect(),
        }
    }
}

pub fn get_sortable_options(
    matches: &mut Vec<SortableLaunchOption>,
    text: &str,
    run_cache_weeks: u8,
    show_execs: bool,
    data_dir: &Path,
) {
    let entries = get_all_desktop_files();
    let runs = get_stored_runs(run_cache_weeks, data_dir);

    if text.is_empty() {
        for entry in entries.iter() {
            matches.push(SortableLaunchOption::from_desktop_entry(
                entry,
                MatchType::Exact,
                &runs,
                show_execs,
            ));
        }
        return;
    }

    let lower_text = text.to_ascii_lowercase();
    for entry in entries.iter() {
        let opt = if entry.name.to_ascii_lowercase().contains(&lower_text) {
            if entry.name.to_ascii_lowercase().starts_with(&lower_text) {
                Some(SortableLaunchOption::from_desktop_entry(
                    entry,
                    MatchType::Exact,
                    &runs,
                    show_execs,
                ))
            } else {
                Some(SortableLaunchOption::from_desktop_entry(
                    entry,
                    MatchType::Name,
                    &runs,
                    show_execs,
                ))
            }
        } else if entry.exec_search.to_ascii_lowercase().contains(&lower_text) {
            if entry
                .exec_search
                .to_ascii_lowercase()
                .starts_with(&lower_text)
            {
                Some(SortableLaunchOption::from_desktop_entry(
                    entry,
                    MatchType::ExecExact,
                    &runs,
                    show_execs,
                ))
            } else {
                Some(SortableLaunchOption::from_desktop_entry(
                    entry,
                    MatchType::ExecName,
                    &runs,
                    show_execs,
                ))
            }
        } else if entry
            .keywords
            .iter()
            .any(|k| k.to_ascii_lowercase().starts_with(&lower_text))
        {
            Some(SortableLaunchOption::from_desktop_entry(
                entry,
                MatchType::Keyword,
                &runs,
                show_execs,
            ))
        } else if entry.type_search.eq(&lower_text) {
            Some(SortableLaunchOption::from_desktop_entry(
                entry,
                MatchType::AppType,
                &runs,
                show_execs,
            ))
        } else {
            None
        };

        // push only if not already in matches
        if let Some(opt) = opt {
            if !matches.iter().any(|m| {
                m.name == opt.name && m.details == opt.details && m.details_long == opt.details_long
            }) {
                matches.push(opt);
            }
        }
    }
}
pub fn launch_option(
    iden: &Option<Box<str>>,
    default_terminal: &Option<Box<str>>,
    data_dir: &Path,
) -> bool {
    let entries = get_all_desktop_files();
    let entry = entries
        .iter()
        .find(|entry| iden.as_deref() == Some(&*entry.source.to_string_lossy()));
    if let Some(entry) = entry {
        let exec = entry.exec.clone();
        run_program(
            &exec,
            entry.exec_path.clone(),
            entry.terminal,
            default_terminal,
        );
        trace!("Saving run: {:?}", entry.source);
        save_run(&entry.source, data_dir).warn("Failed to cache run");
        true
    } else {
        warn!("Failed to find entry for {:?}", iden);
        false
    }
}
