use anyhow::Context;
use chrono::{DateTime, Datelike, Utc};
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

pub fn save_run(desktop_file: &Path, data_dir: &Path) -> anyhow::Result<()> {
    let file = get_current_week(data_dir);
    let mut data = if file.exists() {
        let file = OpenOptions::new()
            .read(true)
            .open(&file)
            .context("Failed to open data file")?;
        from_reader(file).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        // create the file and folder
        std::fs::create_dir_all(file.parent().unwrap())
            .context("Failed to create data directory")?;
        serde_json::json!({})
    };

    data[&*desktop_file.to_string_lossy()] = serde_json::json!(
        data.get(&*desktop_file.to_string_lossy())
            .map_or(1, |v| v.as_i64().unwrap_or(0) + 1)
    );

    trace!("Cache saved to {file:?} (added {:?})", desktop_file);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file)
        .context("Failed to open data file for writing")?;
    serde_json::to_writer_pretty(file, &data).context("Failed to write to data file")?;
    Ok(())
}

fn get_current_week(data_dir: &Path) -> PathBuf {
    let mut path = PathBuf::from(data_dir);
    path.push("runs");
    path.push(get_name_from_timestamp(0));
    path
}

fn get_all_weeks(run_cache_weeks: u8, data_dir: &Path) -> Vec<Box<Path>> {
    let mut weeks = Vec::new();
    for week in 0..run_cache_weeks {
        let mut path = PathBuf::from(data_dir);
        path.push("runs");
        path.push(get_name_from_timestamp(week));
        weeks.push(path.into_boxed_path());
    }
    weeks
}

fn get_name_from_timestamp(week: u8) -> Box<Path> {
    let timestamp = Utc::now().timestamp() - (i64::from(week) * 7 * 24 * 60 * 60);
    let datetime = DateTime::from_timestamp(timestamp, 0).expect("Invalid timestamp");
    Box::from(Path::new(&format!(
        "{}_{}.json",
        datetime.year(),
        datetime.iso_week().week()
    )))
}

pub fn get_stored_runs(run_cache_weeks: u8, data_dir: &Path) -> HashMap<Box<Path>, u64> {
    let mut runs = HashMap::new();

    for week in get_all_weeks(run_cache_weeks, data_dir) {
        let cache_data = if week.exists() {
            match OpenOptions::new().read(true).open(&week) {
                Ok(file) => from_reader(file).unwrap_or_else(|err| {
                    warn!("Failed to open cache file: {:?}", week);
                    debug!("Error: {:?}", err);
                    serde_json::json!({})
                }),
                Err(err) => {
                    warn!("Failed to open cache file: {:?}", week);
                    debug!("Error: {:?}", err);
                    serde_json::json!({})
                }
            }
        } else {
            serde_json::json!({})
        };
        if let Some(obj) = cache_data.as_object() {
            for (path, runs_count) in obj {
                runs.entry(PathBuf::from(path).into_boxed_path())
                    .and_modify(|e| *e += runs_count.as_u64().unwrap_or(0))
                    .or_insert_with(|| runs_count.as_u64().unwrap_or(0));
            }
        } else {
            warn!("Cache data is not an object");
        }
    }
    runs
}
