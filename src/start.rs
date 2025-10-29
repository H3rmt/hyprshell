use crate::keybinds::configure_wm;
use crate::receive_handle::event_handler;
use crate::socket::socket_handler;
use crate::util;
use crate::util::check_new_version;
use adw::gtk::gdk::Display;
use adw::gtk::prelude::*;
use adw::gtk::{
    Application, CssProvider, STYLE_PROVIDER_PRIORITY_USER, glib,
    style_context_add_provider_for_display,
};
use anyhow::Context;
use async_channel::{Receiver, Sender};
use config_lib::Config;
use core_lib::WarnWithDetails;
use core_lib::listener::{
    hyprshell_config_block, hyprshell_config_listener, hyprshell_css_listener,
};
use core_lib::transfer::TransferType;
use exec_lib::listener::{hyprland_config_listener, monitor_listener};
use exec_lib::{info_toast, toast};
use launcher_lib::{LauncherData, create_windows_overview_launcher_window};
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::{env, process, thread};
use tracing::{debug, debug_span, error, info, trace, warn};
use windows_lib::{
    WindowsOverviewData, WindowsSwitchData, create_windows_overview_window,
    create_windows_switch_window,
};

const NEW_VERSION_INFO: &str = "This version uses a hyprland plugin to register keypresses, adds shell completion, improves UI, adds usefull commands and much more.";

pub fn start(
    config_path: PathBuf,
    css_path: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
) -> anyhow::Result<()> {
    let config_path = Rc::new(config_path);
    let css_path = Rc::new(css_path);
    let data_dir = Rc::new(data_dir);
    let cache_dir = Rc::new(cache_dir);

    util::preactivate().context("Failed to preactivate GTK and reload icons")?;
    exec_lib::reload_hyprland_config()
        .context("Failed to reload hyprland config")
        .warn_details("unable to reload hyprland config");

    let (event_sender, event_receiver) = async_channel::unbounded();

    if env::var_os("HYPRSHELL_NO_LISTENERS").is_none() {
        register_event_restarter(config_path.clone(), css_path.clone(), event_sender.clone());
    }

    let event_sender_2 = event_sender.clone();
    thread::spawn(move || {
        socket_handler(&event_sender_2);
    });

    let wayland_socket_index = env::var("WAYLAND_DISPLAY")
        .ok()
        .and_then(|s| s.split('-').next_back()?.parse::<i32>().ok())
        .unwrap_or(1);

    info!("Starting gui loop");
    loop {
        let application = Application::builder()
            .application_id(format!(
                "{}-{}{}",
                core_lib::APPLICATION_ID,
                wayland_socket_index,
                if cfg!(debug_assertions) { "-test" } else { "" }
            ))
            .build();
        debug!("Application created");

        let config_path = config_path.clone();
        let css_path = css_path.clone();
        let data_dir = data_dir.clone();
        let cache_dir = cache_dir.clone();
        let event_sender = event_sender.clone();
        let event_receiver = event_receiver.clone();
        application.connect_activate(move |app| {
            activate(
                app,
                &config_path,
                &css_path,
                &data_dir,
                &cache_dir,
                event_sender.clone(),
                event_receiver.clone(),
            );
        });
        let exit = application.run_with_args::<String>(&[]);
        debug!("Application exited with code {exit:?}");
    }
}

pub struct Globals {
    pub windows: Option<WindowsGlobal>,
    pub app: Application,
}

#[derive(Debug, Default)]
pub struct WindowsGlobal {
    pub overview: Option<(WindowsOverviewData, LauncherData)>,
    pub switch: Option<WindowsSwitchData>,
}

#[allow(clippy::cognitive_complexity)]
fn activate(
    app: &Application,
    config_path: &Path,
    css_path: &Path,
    data_dir: &Path,
    cache_dir: &Path,
    event_sender: Sender<TransferType>,
    event_receiver: Receiver<TransferType>,
) {
    let _span = debug_span!("activate").entered();
    apply_css(css_path).warn_details("Failed to apply CSS");
    exec_lib::set_follow_mouse_default().warn_details("Failed to set set_remain_focused default");
    match check_new_version(cache_dir) {
        Err(err) => {
            debug!("Unable to compare previous to current version.\n{err:?}");
        }
        Ok((Ordering::Greater, no_patch)) => {
            info_toast(
                &format!(
                    "Hyprshell was updated to a new version ({})",
                    env!("CARGO_PKG_VERSION")
                ),
                Duration::from_secs(5),
            );
            if no_patch {
                info_toast(NEW_VERSION_INFO, Duration::from_secs(10));
            }
        }
        Ok((Ordering::Less, _)) => {
            toast("Hyprshell was downgraded, downgrading config must be done manually if needed");
        }
        Ok((Ordering::Equal, _)) => {
            debug!("Hyprshell is up to date");
        }
    }

    let config = match config_lib::load_and_migrate_config(config_path, true) {
        Ok(config) => config,
        Err(err) => {
            error!(
                "Failed to load config from path {}: {err:?}",
                config_path.display()
            );
            toast(&format!("Failed to load config: {err:?}"));
            if let Err(err) = hyprshell_config_block(config_path) {
                error!("Failed to block config: {err:?}",);
                process::exit(1);
            }
            info!("Trying to rerun application after config reload");
            return; // return needed to exit the application
        }
    };

    // TODO remove in future if more is available
    if config.windows.is_none()
        || matches!(&config.windows, Some(windows) if windows.overview.is_none() && windows.switch.is_none())
    {
        warn!("Nothing is enabled in the config");
        toast("Nothing is enabled in the config");
        if let Err(err) = hyprshell_config_block(config_path) {
            error!("Failed to block config: {err:?}",);
            process::exit(1);
        }
        info!("Trying to rerun application after config reload");
        return; // return needed to exit the application
    }

    if let Err(err) = configure_wm(&config) {
        error!("Failed to create keybinds: {err:?}");
        toast(&format!("Failed to create keybinds: {err:?}"));
        if let Err(err) = hyprshell_config_block(config_path) {
            error!("Failed to block config: {err:?}");
            process::exit(1);
        }
        info!("Trying to rerun application after config reload");
        return; // return needed to exit the application
    }

    let globals = match create_windows(app, &config, data_dir, event_sender.clone()) {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to create windows: {err:?}");
            toast(&format!("Failed to create windows: {err:?}"));
            if let Err(err) = hyprshell_config_block(config_path) {
                error!("Failed to block config: {err:?}");
                process::exit(1);
            }
            info!("Trying to rerun application after config reload");
            return; // return needed to exit the application
        }
    };

    glib::spawn_future_local(async move {
        event_handler(globals, event_receiver, event_sender).await;
        info!("Application exited, restarting");
    });

    info!("Application initialized");
}

fn create_windows(
    app: &Application,
    config: &Config,
    data_dir: &Path,
    event_sender: Sender<TransferType>,
) -> anyhow::Result<Globals> {
    let mut global = Globals {
        windows: None,
        app: app.clone(),
    };
    if let Some(windows) = &config.windows {
        let mut windows_data = WindowsGlobal::default();
        if let Some(overview) = &windows.overview {
            let overview_data = create_windows_overview_window(app, overview, windows)
                .context("failed to create overview window")?;
            let launcher_data = create_windows_overview_launcher_window(
                app,
                &overview.launcher,
                data_dir,
                &event_sender,
            )
            .context("failed to create launcher window")?;
            windows_data.overview = Some((overview_data, launcher_data));
        } else {
            debug!("Windows overview disabled");
        }
        if let Some(switch) = &windows.switch {
            let switch_data = create_windows_switch_window(app, switch, windows, event_sender)
                .context("failed to create switch window")?;
            windows_data.switch = Some(switch_data);
        } else {
            debug!("Windows switch disabled");
        }
        global.windows = Some(windows_data);
    } else {
        debug!("Windows disabled");
    }
    Ok(global)
}

fn apply_css(custom_css: &Path) -> anyhow::Result<()> {
    let provider_app = CssProvider::new();
    provider_app.load_from_bytes(&glib::Bytes::from_static(include_bytes!(
        "default_styles.css"
    )));
    style_context_add_provider_for_display(
        &Display::default().context("Could not connect to a display.")?,
        &provider_app,
        STYLE_PROVIDER_PRIORITY_USER,
    );

    windows_lib::get_css()?;
    launcher_lib::get_css()?;

    if custom_css.exists() {
        debug!("Loading custom css file {custom_css:?}");
        let provider_user = CssProvider::new();
        provider_user.load_from_path(custom_css);
        style_context_add_provider_for_display(
            &Display::default().context("Could not connect to a display.")?,
            &provider_user,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    } else {
        debug!("Custom css file {custom_css:?} does not exist");
    }
    Ok(())
}

pub fn register_event_restarter(
    config_path: Rc<PathBuf>,
    css_path: Rc<PathBuf>,
    event_sender: Sender<TransferType>,
) {
    // delay for 1.5 seconds to allow the config to be reloaded before listening for reload
    let delay = env::var("HYPRSHELL_RELOAD_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1500);
    let (restart_sender, restart_receiver) = async_channel::unbounded();
    glib::timeout_add_local_once(Duration::from_millis(delay), move || {
        setup_restart_listener(&config_path, &css_path, &restart_sender);
    });

    // State to track the current debounce timer
    let debounce_timer = Rc::new(RefCell::new(None::<glib::SourceId>));
    glib::spawn_future_local(async move {
        loop {
            let cause = restart_receiver.recv().await.unwrap_or_default();
            debug!("Received restart request ({cause}), starting debounce timer");

            // Cancel any existing timer
            if let Some(timer_id) = debounce_timer.borrow_mut().take() {
                timer_id.remove();
                trace!("Cancelled previous debounce timer");
            }

            // Create new debounce timer
            let event_sender_clone = event_sender.clone();
            let debounce_timer_clone = debounce_timer.clone();
            let timer_id = glib::timeout_add_local_once(Duration::from_millis(delay), move || {
                trace!("Debounce timer expired, triggering restart ({cause})");

                // Clear the timer reference since it's about to complete
                *debounce_timer_clone.borrow_mut() = None;

                // Send the restart event
                let event_sender_inner = event_sender_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Restarting gui ({cause})");
                    event_sender_inner
                        .send(TransferType::Restart)
                        .await
                        .warn_details("unable to send restart");
                });
            });

            // Store the timer ID so we can cancel it if needed
            *debounce_timer.borrow_mut() = Some(timer_id);
        }
    });
}

static WATCHERS: OnceLock<Mutex<Vec<Box<dyn Any + Send>>>> = OnceLock::new();

fn setup_restart_listener(config_path: &Path, css_path: &Path, restart_tx: &Sender<&'static str>) {
    let tx = restart_tx.clone();
    if let Ok(watcher) = hyprshell_config_listener(config_path, move |mess| {
        let _ = tx.send_blocking(mess);
    }) {
        WATCHERS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("Failed to lock watchers")
            .push(Box::new(watcher));
    }
    let tx = restart_tx.clone();
    if let Ok(watcher) = hyprshell_css_listener(css_path, move |mess| {
        let _ = tx.send_blocking(mess);
    }) {
        WATCHERS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("Failed to lock watchers")
            .push(Box::new(watcher));
    }

    let tx = restart_tx.clone();
    glib::spawn_future_local(async move {
        monitor_listener(move |mess| {
            let _ = tx.send_blocking(mess);
        })
        .await;
    });
    // only listen if plugin is not used
    if env::var_os("HYPRSHELL_NO_USE_PLUGIN").is_some() {
        let tx = restart_tx.clone();
        glib::spawn_future_local(async move {
            hyprland_config_listener(move |mess| {
                let _ = tx.send_blocking(mess);
            })
            .await;
        });
    }
}
