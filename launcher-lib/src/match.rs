use crate::data::get_cached_runs;
use crate::desktop_map::{get_all_desktop_files, DesktopEntry};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::trace;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MatchType {
    Other = 1,
    Keyword = 5,
    Name = 10,
    Exact = 15,
}

#[derive(Debug)]
pub struct Match {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub exec: Box<str>,
    pub exec_path: Option<Box<Path>>,
    pub terminal: bool,
    pub source: Option<Box<Path>>,
}
impl From<DesktopEntry> for Match {
    fn from(entry: DesktopEntry) -> Self {
        Match {
            name: entry.name,
            icon: entry.icon,
            exec: entry.exec,
            exec_path: entry.exec_path,
            terminal: entry.terminal,
            source: Some(entry.source),
        }
    }
}

fn compare_matches(
    runs: HashMap<Box<Path>, i64>,
) -> impl Fn(&(MatchType, DesktopEntry), &(MatchType, DesktopEntry)) -> Ordering {
    move |a: &(MatchType, DesktopEntry), b: &(MatchType, DesktopEntry)| {
        // trace!(
        //     "{} |{:?}|({:?}) / {} |{:?}|({:?})",
        //     b.0 as i64 + runs.get(&b.1.path).unwrap_or(&0),
        //     b.0,
        //     b.1,
        //     a.0 as i64 + runs.get(&a.1.path).unwrap_or(&0),
        //     a.0,
        //     a.1
        // );
        // sort in reverse order
        match (
            b.0 as i64 + runs.get(&b.1.source).unwrap_or(&0),
            a.0 as i64 + runs.get(&a.1.source).unwrap_or(&0),
        ) {
            (a1, b1) if a1 > b1 => Ordering::Greater,
            (a1, b1) if a1 < b1 => Ordering::Less,
            (a1, b1) if a1 == b1 => {
                // sort by name
                a.1.name.cmp(&b.1.name)
            }
            _ => unreachable!(),
        }
    }
}

pub fn get_matches(
    text: &str,
    launcher_max_items: usize,
    run_cache_weeks: u8,
    show_shell: bool,
    data_dir: &Path,
) -> Vec<(MatchType, Match)> {
    let entries = get_all_desktop_files();
    let mut matches = HashMap::new();
    for entry in entries.iter() {
        if entry.keywords.iter().any(|k| {
            k.to_ascii_lowercase()
                .starts_with(&text.to_ascii_lowercase())
        }) {
            matches.insert(entry.source.clone(), (MatchType::Keyword, entry.clone()));
        }
    }
    // do name last to let them appear first
    for entry in entries.iter() {
        if entry
            .name
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase())
            || entry
                .exec
                .to_ascii_lowercase()
                .contains(&text.to_ascii_lowercase())
        {
            if entry
                .name
                .to_ascii_lowercase()
                .starts_with(&text.to_ascii_lowercase())
                || entry
                    .exec
                    .to_ascii_lowercase()
                    .starts_with(&text.to_ascii_lowercase())
            {
                matches.insert(entry.source.clone(), (MatchType::Exact, entry.clone()));
            } else {
                matches.insert(entry.source.clone(), (MatchType::Name, entry.clone()));
            }
        }
    }
    let runs = get_cached_runs(run_cache_weeks, data_dir);

    // sort each of the sections by times run in the past
    let mut matches: Vec<_> = matches.into_values().collect();
    matches.sort_by(compare_matches(runs));
    let mut matches = matches
        .into_iter()
        .take(launcher_max_items)
        .map(|(t, e)| (t, Match::from(e)))
        .collect::<Vec<_>>();

    if show_shell {
        matches.push((
            MatchType::Other,
            Match {
                name: "Run in shell".into(),
                icon: Some(PathBuf::from("system-run").into_boxed_path()),
                exec: Box::from(text),
                terminal: false,
                exec_path: None,
                source: None,
            },
        ));
        matches.push((
            MatchType::Name,
            Match {
                name: "Run in terminal".into(),
                icon: Some(PathBuf::from("utilities-terminal").into_boxed_path()),
                exec: Box::from(format!("$SHELL -c \"{text}; exec $SHELL\"")),
                terminal: true,
                exec_path: None,
                source: None,
            },
        ))
    }

    trace!(
        "Matches: {:?}",
        matches
            .iter()
            .map(|(v, e)| format!("{:?}: {}", v, e.name))
            .collect::<Vec<_>>()
    );
    matches
}
