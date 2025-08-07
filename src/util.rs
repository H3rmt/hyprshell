use anyhow::Context;
use core_lib::{Warn, WarnWithDetails, default};
use gtk::glib::ControlFlow;
use gtk::{IconTheme, Settings, glib};
use signal_hook::consts::{SIGINT, SIGKILL, SIGSTOP, SIGTERM};
use signal_hook::iterator::Signals;
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use tracing::{info, trace, warn};

pub fn preactivate() {
    let _span = tracing::span!(tracing::Level::TRACE, "preactivate").entered();
    handle_sigterm();

    init_gtk();
    check_themes();

    reload_icons(true);
    reload_desktop_data();
}

pub fn reload_icons(background: bool) {
    if let Some(data) = get_icon_data().context("Failed to reload GTK icons").warn() {
        default::reload_available_icons(data.0, data.1, background)
            .context("Failed to reload GTK icons")
            .warn();
    } else {
        warn!("Failed to get GTK icon data, skipping icon reload");
    }
}

/// TODO run this after each launcher open async
pub fn reload_desktop_data() {
    default::reload_default_files();
    windows_lib::reload_class_to_icon_map();
    launcher_lib::reload_applications_desktop_entries_map();
}

pub fn init_gtk() {
    gtk::init().expect("Failed to initialize GTK");
}

pub(crate) fn check_themes() {
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
    let Ok(mut signals) = Signals::new(&[SIGTERM, SIGINT]) else {
        warn!("Failed to create signal handler for SIGTERM and SIGINT");
        return;
    };
    thread::spawn(move || {
        for sig in signals.forever() {
            info!("Received sig: {sig}, exiting gracefully");
            exec_lib::reset_remain_focused()
                .warn_details("Failed to reset follow mouse on SIGTERM");
            // Continue with the default SIGTERM handler after cleanup
            exit(0);
        }
    });
}

fn get_icon_data() -> anyhow::Result<(Vec<String>, Vec<PathBuf>)> {
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
    Ok((gtk_icons, search_path))
}
