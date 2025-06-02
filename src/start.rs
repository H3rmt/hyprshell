use crate::keybinds::create_binds_and_submaps;
use crate::receive::{Globals, socket_handler};
use core_lib::theme_icon_cache::init_icon_map;
use core_lib::transfer::TransferType;
use core_lib::{
    Warn, collect_desktop_files, config, hyprshell_config_block, hyprshell_config_listener,
    hyprshell_css_listener, send_to_socket,
};
use exec_lib::listener::{hyprland_config_listener, monitor_listener};
use exec_lib::{apply_keybinds, reload_config, toast};
use gtk::gdk::Display;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{
    Application, CssProvider, IconTheme, STYLE_PROVIDER_PRIORITY_APPLICATION,
    STYLE_PROVIDER_PRIORITY_USER, Settings, glib, style_context_add_provider_for_display,
};
use launcher_lib::{LauncherGlobal, create_launcher_window};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{Level, debug, info, span, trace, warn};
use windows_lib::{WindowsGlobal, create_windows_window};

const APPLICATION_ID: &str = "com.github.h3rmt.hyprshell";

pub fn start(config_path: PathBuf, css_path: PathBuf, data_dir: PathBuf) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "start").entered();
    loop {
        let application = Application::builder()
            .application_id(&if cfg!(debug_assertions) {
                format!("{}-debug", APPLICATION_ID)
            } else {
                APPLICATION_ID.to_string()
            })
            .build();

        let config_path = config_path.clone();
        let css_path = css_path.clone();
        let data_dir = data_dir.clone();
        application.connect_activate(move |app| activate(app, &config_path, &css_path, &data_dir));
        application.run_with_args::<String>(&[]);
    }
}
fn activate(app: &Application, config_path: &Path, css_path: &Path, data_dir: &Path) {
    let _span = span!(Level::TRACE, "activate").entered();
    // reloading the config is needed to delete the old submaps
    reload_config();

    check_themes();
    fill_icon_map(true);

    let desktop_files = collect_desktop_files();
    windows_lib::reload_desktop_map(&desktop_files);
    launcher_lib::reload_applications_desktop_map(&desktop_files);
    launcher_lib::reload_search_default_browser(&desktop_files);

    let config = match config::load_config(config_path) {
        Err(err) => {
            toast(&format!("Failed to load config: {:?}", err));
            hyprshell_config_block(config_path);
            return;
        }
        Ok(config) => config,
    };

    if let Ok(keybinds) = create_binds_and_submaps(&config) {
        apply_keybinds(keybinds);
    } else {
        warn!("Failed to apply keybinds");
        toast("Failed to create keybinds");
    }
    apply_css(css_path);

    let windows_data: Option<WindowsGlobal> = config.windows.map(WindowsGlobal::new);
    let mut launcher_data: Option<LauncherGlobal> =
        config.launcher.map(LauncherGlobal::new(data_dir));

    if let Some(windows_data) = &windows_data {
        create_windows_window(app, windows_data)
            .unwrap_or_else(|e| toast(&format!("Failed to create windows window(s): {e}")));
    } else {
        debug!("Overview is disabled");
    }

    if let Some(launcher_data) = &mut launcher_data {
        create_launcher_window(app, launcher_data)
            .unwrap_or_else(|e| toast(&format!("Failed to create launcher window: {e}")));
    } else {
        debug!("Launcher is disabled");
    }

    let globals = Globals {
        window: windows_data,
        launcher: launcher_data,
    };

    let config_path = PathBuf::from(config_path);
    let css_path = PathBuf::from(css_path);
    glib::spawn_future_local(async move {
        if env::var_os("HYPRSHELL_NO_LISTENERS").is_none() {
            restart_listener(config_path, css_path).await;
        }
    });

    glib::spawn_future_local(async move {
        socket_handler(globals).await;
    });

    debug!("Application initialized");
}

async fn restart_listener(config_path: PathBuf, css_path: PathBuf) {
    let (tx, rx) = async_channel::bounded(1);
    glib::spawn_future_local(clone!(
        #[strong]
        tx,
        async move {
            let (tx2, rx2) = async_channel::bounded(1);
            // must be kept in scope to keep the watcher alive
            let _watcher = hyprshell_config_listener(&config_path, move |mess| {
                let _ = tx.send_blocking(mess);
                let _ = tx2.send_blocking(mess);
            });
            // watcher.watch doesnt block so we block
            let _ = rx2.recv().await;
        }
    ));
    glib::spawn_future_local(clone!(
        #[strong]
        tx,
        async move {
            let (tx2, rx2) = async_channel::bounded(1);
            // must be kept in scope to keep the watcher alive
            let _watcher = hyprshell_css_listener(&css_path, move |mess| {
                let _ = tx.send_blocking(mess);
                let _ = tx2.send_blocking(mess);
            });
            // watcher.watch doesnt block so we block
            let _ = rx2.recv().await;
        }
    ));
    glib::spawn_future_local(clone!(
        #[strong]
        tx,
        async move {
            monitor_listener(move |mess| {
                let _ = tx.send_blocking(mess);
            })
            .await;
        }
    ));
    glib::timeout_add_local_once(
        // delay for 1 second to allow the config to be reloaded before listening for reload
        Duration::from_secs(1),
        || {
            glib::spawn_future_local(async move {
                hyprland_config_listener(move |mess| {
                    let _ = tx.send_blocking(mess);
                })
                .await;
            });
        },
    );
    let cause = rx.recv().await.unwrap_or_default();
    info!("Restarting gui ({cause})");
    send_to_socket(&TransferType::Restart).warn("unable to send restart");
}

fn apply_css(custom_css: &Path) {
    let provider_app = CssProvider::new();
    provider_app.load_from_bytes(&glib::Bytes::from_static(include_bytes!(
        "default-styles.css"
    )));
    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    windows_lib::get_css();
    launcher_lib::get_css();

    if !custom_css.exists() {
        info!("Custom css file {custom_css:?} does not exist");
    } else {
        debug!("Loading custom css file {custom_css:?}");
        let provider_user = CssProvider::new();
        provider_user.load_from_path(custom_css);
        style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider_user,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

fn check_themes() {
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

pub fn fill_icon_map(threads: bool) {
    gtk::init().expect("Failed to initialize GTK");

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
    init_icon_map(gtk_icons, search_path, threads);
}
