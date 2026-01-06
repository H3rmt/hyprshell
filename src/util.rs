use anyhow::Context;
use core_lib::{Warn, WarnWithDetails, default};
use relm4::adw::gtk::{IconTheme, Settings};
use relm4::gtk;
use semver::Version;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::cmp::Ordering;
use std::fs::{File, read_to_string, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, thread, time};
use tracing::{debug, info, trace, warn};

pub fn preactivate() -> anyhow::Result<()> {
    let _span = tracing::span!(tracing::Level::TRACE, "preactivate").entered();
    handle_sigterm();

    init_gtk();
    check_themes();

    reload_icons(true);
    reload_desktop_data().context("Failed to reload desktop data")?;
    Ok(())
}

pub fn reload_icons(background: bool) {
    let data = get_icon_data();
    default::reload_available_icons(data.0, data.1, background)
        .context("Failed to reload GTK icons")
        .warn();
}

pub fn reload_desktop_data() -> anyhow::Result<()> {
    let start = time::Instant::now();
    default::reload_default_files().context("Failed to reload default files")?;
    windows_lib::reload_class_to_icon_map().context("Failed to reload class to icon map")?;
    launcher_lib::reload_applications_desktop_entries_map()
        .context("Failed to reload desktop entries")?;
    debug!("Reloaded desktop data in {:?}", start.elapsed());
    Ok(())
}

pub fn init_gtk() {
    gtk::init().expect("Failed to initialize GTK");
}

pub fn check_themes() {
    if let Some(settings) = Settings::default() {
        let theme_name = settings.gtk_theme_name();
        let icon_theme_name = settings.gtk_icon_theme_name();
        info!(
            "Using theme: {theme_name:?} and icon theme: {icon_theme_name:?}, please make sure both exist, else weird icon or graphical issues may occur"
        );
    } else {
        warn!("Unable to check default settings for icon theme");
    }
}

fn handle_sigterm() {
    let Ok(mut signals) = Signals::new([SIGTERM, SIGINT]) else {
        warn!("Failed to create signal handler for SIGTERM and SIGINT");
        return;
    };
    thread::spawn(move || {
        if let Some(sig) = signals.forever().next() {
            info!("Received sig: {sig}, exiting gracefully");
            exec_lib::reset_no_follow_mouse()
                .warn_details("Failed to reset follow mouse on SIGTERM");
            if let Err(err) = exec_lib::plugin::unload() {
                warn!("Failed to unload plugin: {err:?}",);
            }
            exit(0);
        }
    });
}

fn get_icon_data() -> (Vec<String>, Vec<PathBuf>) {
    let icon_theme = IconTheme::new();
    let gtk_icons = icon_theme
        .icon_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let search_path = icon_theme
        .search_path()
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    trace!("Icon theme search path: {search_path:?}");
    (gtk_icons, search_path)
}

const NEW_VERSION_INFOS: &[(&str, &str)] = &[
    (
        "4.7.0",
        "Version 4.7.0 adds a hyprland plugin to register keypresses, adds shell completion, improves UI, adds usefull commands and much more.",
    ),
    (
        "4.8.0",
        "Version 4.8.0 adds a graphical settings Editor, support for special workspaces and vim motions",
    ),
    (
        "4.9.0",
        "Version 4.9.0 adds a new Settings Editor, replaces tui config generation",
    ),
];

/// Checks if the current version is newer than the cached version.
/// If a mayor of minor update is detected, true is returned as second value.
pub fn check_new_version(cache_dir: &Path) -> anyhow::Result<(Ordering, Vec<String>)> {
    let version_file = cache_dir.join("last_version.txt");
    let current_version = env!("CARGO_PKG_VERSION");
    let current_version =
        Version::parse(current_version).context("Failed to parse current version")?;
    let cached_version = if version_file.exists() {
        let contents = read_to_string(&version_file).context("Failed to read old version file")?;
        Version::parse(contents.trim()).context("Failed to parse old version")?
    } else {
        fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;
        let mut file = File::create(&version_file).context("Failed to create version file")?;
        file.write_all(current_version.to_string().as_bytes())
            .context("Failed to write current version to file")?;
        Version::new(0, 0, 0)
    };
    trace!(
        "Cached version: {cached_version:?}, current version: {current_version:?}: {:?}",
        cached_version.cmp(&current_version)
    );
    write(&version_file, current_version.to_string().as_bytes())
        .context("Failed to write current version to file")?;
    let versions = filter_version_messages(NEW_VERSION_INFOS, &current_version, &cached_version);
    Ok((current_version.cmp(&cached_version), versions))
}

fn filter_version_messages(
    versions: &[(&str, &str)],
    current_version: &Version,
    cached_version: &Version,
) -> Vec<String> {
    if current_version <= cached_version {
        return Vec::new();
    }
    let mut parsed: Vec<(Version, String)> = versions
        .iter()
        .filter_map(|(k, s)| {
            let ver_str = (*k).to_string();
            Version::parse(&ver_str)
                .ok()
                .warn_details("version in NEW_VERSION_INFOS has invalid semver")
                .map(|v| (v, (*s).to_string()))
        })
        .collect();
    parsed.sort_by(|a, b| a.0.cmp(&b.0));
    parsed
        .into_iter()
        .filter_map(|(v, s)| {
            if v > *cached_version && v <= *current_version {
                Some(s)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::filter_version_messages;
    use semver::Version;

    fn v(s: &str) -> Version {
        Version::parse(s).expect("invalid version")
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_empty_versions() {
        let result = filter_version_messages(&[], &v("1.0.0"), &v("0.9.0"));
        assert!(result.is_empty());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_older_current_version() {
        let versions = [("1.0.0", "test")];
        let result = filter_version_messages(&versions, &v("0.9.0"), &v("1.0.0"));
        assert!(result.is_empty());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_newer_current_version() {
        let versions = [("1.0.0", "1.0.0 test")];
        let result = filter_version_messages(&versions, &v("1.0.0"), &v("0.9.0"));
        assert_eq!(result, vec!["1.0.0 test"]);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_same_version() {
        let versions = [("1.0.0", "test")];
        let result = filter_version_messages(&versions, &v("1.0.0"), &v("1.0.0"));
        assert!(result.is_empty());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_multiple_versions() {
        let versions = [
            ("1.0.0", "1.0.0 test1"),
            ("2.0.0", "2.0.0 test2"),
            ("1.5.0", "1.5.0 test3"),
        ];
        let result = filter_version_messages(&versions, &v("2.0.0"), &v("1.0.0"));
        assert_eq!(result, vec!["1.5.0 test3", "2.0.0 test2"]);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_all_versions() {
        let versions = [
            ("1.0.0", "1.0.0 test1"),
            ("2.0.0", "2.0.0 test2"),
            ("1.5.0", "1.5.0 test3"),
        ];
        let result = filter_version_messages(&versions, &v("2.0.0"), &v("0.0.0"));
        assert_eq!(result, vec!["1.0.0 test1", "1.5.0 test3", "2.0.0 test2"]);
    }
}
