use anyhow::Context;
use core_lib::{Warn, WarnWithDetails, default};
use gtk::{IconTheme, Settings};
use semver::Version;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::cmp::Ordering;
use std::fs::{File, read_to_string, write};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::thread;
use tracing::{info, trace, warn};

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

/// TODO run this after each launcher open async
pub fn reload_desktop_data() -> anyhow::Result<()> {
    default::reload_default_files().context("Failed to reload default files")?;
    windows_lib::reload_class_to_icon_map().context("Failed to reload class to icon map")?;
    launcher_lib::reload_applications_desktop_entries_map()
        .context("Failed to reload desktop entries")?;
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
            exec_lib::reset_remain_focused()
                .warn_details("Failed to reset follow mouse on SIGTERM");
            if let Err(err) = exec_lib::plugin::unload() {
                warn!("Failed to unload plugin: {err}");
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

pub fn check_new_version(cache_dir: &Path) -> anyhow::Result<Ordering> {
    let version_file = cache_dir.join("version.txt");
    let current_version = env!("CARGO_PKG_VERSION");
    let current_version =
        Version::parse(current_version).context("Failed to parse current version")?;
    if version_file.exists() {
        let contents = read_to_string(&version_file).context("Failed to read old version file")?;
        let cached_version =
            Version::parse(contents.trim()).context("Failed to parse old version")?;
        trace!(
            "Cached version: {cached_version:?}, current version: {current_version:?}: {:?}",
            cached_version.cmp(&current_version)
        );
        write(&version_file, current_version.to_string().as_bytes())
            .context("Failed to write current version to file")?;
        Ok(current_version.cmp(&cached_version))
    } else {
        std::fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;
        let mut file = File::create(&version_file).context("Failed to create version file")?;
        file.write_all(current_version.to_string().as_bytes())
            .context("Failed to write current version to file")?;
        Ok(Ordering::Greater)
    }
}
