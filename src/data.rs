use std::fs::read_to_string;
use std::path::Path;
use tracing::debug;

pub(crate) fn launch_history(
    run_cache_weeks: Option<u8>,
    config_path: &Path,
    data_dir: &Path,
    verbose: u8,
) {
    let run_cache_weeks = run_cache_weeks.unwrap_or_else(|| {
        config_lib::load_and_migrate_config(config_path, true)
            .ok()
            .and_then(|c| {
                c.windows.and_then(|w| {
                    w.overview
                        .map(|o| o.launcher)
                        .and_then(|l| l.plugins.applications.as_ref().map(|a| a.run_cache_weeks))
                })
            })
            .unwrap_or(4)
    });
    debug!("showing history for the last {} weeks", run_cache_weeks);

    let runs = launcher_lib::get_applications_stored_runs(run_cache_weeks, data_dir);
    let mut sorted = runs.into_iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (path, run) in sorted {
        // ignore the ini parser for this, just read the file and find, is faster
        if let Ok(content) = read_to_string(&path) {
            if let Some(name_line) = content.lines().find(|l| l.starts_with("Name=")) {
                let name = name_line.trim_start_matches("Name=");
                // check if verbosity is set, if so, print the name
                if verbose > 0 {
                    println!("{name}: ({run}) {}", path.display());
                } else if verbose == 0 {
                    println!("{name}: ({run})");
                }
            } else {
                println!("?{}?: ({run})", path.display());
            }
        } else {
            println!("?{}?: ({run})", path.display());
        }
    }
}
