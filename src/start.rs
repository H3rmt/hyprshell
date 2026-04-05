use crate::receive_handle::event_handler;
use crate::root::{Root, RootInit};
use crate::socket::socket_handler;
use crate::util;
use crate::util::check_new_version;
use crate::wm::{configure_wm, configure_wm_initial};
use anyhow::{Context, bail};
use async_channel::{Receiver, Sender};
use config_lib::Config;
use core_lib::listener::{
    hyprshell_config_block, hyprshell_config_listener, hyprshell_css_listener,
};
use core_lib::transfer::TransferType;
use core_lib::{WarnWithDetails, notify, notify_resident, notify_warn};
use exec_lib::listener::{hyprland_config_listener, monitor_listener};
use launcher_lib::{LauncherData, create_windows_overview_launcher_window};
use relm4::RelmApp;
use relm4::adw::gio;
use relm4::adw::gtk::gdk::Display;
use relm4::adw::gtk::prelude::*;
use relm4::adw::gtk::{
    Application, CssProvider, STYLE_PROVIDER_PRIORITY_USER, glib,
    style_context_add_provider_for_display,
};
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::{env, process, thread};
use tracing::{debug, debug_span, error, info, trace};
use windows_lib::{WindowsOverviewData, WindowsSwitchData};

pub fn start(
    config_file: PathBuf,
    css_path: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
) -> anyhow::Result<()> {
    let config_file = Rc::new(config_file);
    let css_path = Rc::new(css_path);
    let data_dir = Rc::new(data_dir);
    let cache_dir = Rc::new(cache_dir);

    util::preactivate(&cache_dir).context("Failed to preactivate GTK")?;
    match check_new_version(&cache_dir) {
        Err(err) => {
            debug!("Unable to compare previous to current version.\n{err:?}");
        }
        Ok((Ordering::Greater, messages)) => {
            notify(
                &format!(
                    "Hyprshell was updated to a new version ({})",
                    env!("CARGO_PKG_VERSION")
                ),
                Duration::from_secs(5),
            );
            thread::sleep(Duration::from_millis(500));
            for info in messages {
                notify_resident(&info, Duration::from_secs(10));
            }
        }
        Ok((Ordering::Less, _)) => {
            notify_warn(
                "Hyprshell was downgraded, downgrading config must be done manually if needed",
            );
        }
        Ok((Ordering::Equal, _)) => {
            debug!("Hyprshell version did not change");
        }
    }

    // TODO
    // if !configure_wm_initial(&cache_dir) {
    //     bail!("Failed to configure wm");
    // }

    let wayland_socket_index = env::var("WAYLAND_DISPLAY")
        .ok()
        .and_then(|s| s.split('-').next_back()?.parse::<i32>().ok())
        .unwrap_or(1);
    let id = format!(
        "{}-{}-{}",
        core_lib::APPLICATION_ID,
        wayland_socket_index,
        if cfg!(debug_assertions) { "-test" } else { "" }
    );
    trace!("Application id: {}", id);
    let relm = RelmApp::new(&id)
        .visible_on_activate(false)
        .with_args(vec![]);
    debug!("Application created");

    apply_css(&css_path).warn_details("Failed to apply CSS");

    relm.run::<Root>(RootInit {});
    Ok(())
}

pub struct Globals {
    pub windows: Option<WindowsGlobal>,
    pub app: Application,
    pub active: bool,
}

#[derive(Debug, Default)]
pub struct WindowsGlobal {
    // bool ise used to indicate that the launcher is active
    pub overview: Option<(WindowsOverviewData, LauncherData, bool)>,
    pub switch: Option<WindowsSwitchData>,
}

#[allow(clippy::cognitive_complexity)]
fn activate(
    app: &Application,
    config_file: &Path,
    css_path: &Path,
    data_dir: &Path,
    cache_dir: &Path,
    event_sender: Sender<TransferType>,
    event_receiver: Receiver<TransferType>,
) {
    let _span = debug_span!("activate").entered();
    apply_css(css_path).warn_details("Failed to apply CSS");
    // exec_lib::set_follow_mouse_default().warn_details("Failed to set set_remain_focused default");

    let config = match config_lib::load_and_migrate_config(config_file, true) {
        Ok(config) => config,
        Err(err) => {
            notify_warn(&format!("Failed to load config: {err:?}"));
            if let Err(err) = hyprshell_config_block(config_file) {
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
        notify_warn("Nothing is enabled in the config");
        if let Err(err) = hyprshell_config_block(config_file) {
            error!("Failed to block config: {err:?}",);
            process::exit(1);
        }
        info!("Trying to rerun application after config reload");
        return; // return needed to exit the application
    }

    if let Err(err) = configure_wm(&config, cache_dir) {
        notify_warn(&format!("Failed to configure wm: {err:?}"));
        if let Err(err) = hyprshell_config_block(config_file) {
            error!("Failed to block config: {err:?}");
            process::exit(1);
        }
        info!("Trying to rerun application after config reload");
        return; // return needed to exit the application
    }

    let globals = match create_windows(app, &config, data_dir, event_sender.clone()) {
        Ok(data) => data,
        Err(err) => {
            notify_warn(&format!("Failed to create windows: {err:?}"));
            if let Err(err) = hyprshell_config_block(config_file) {
                error!("Failed to block config: {err:?}");
                process::exit(1);
            }
            info!("Trying to rerun application after config reload");
            return; // return needed to exit the application
        }
    };

    event_sender
        .send_blocking(TransferType::SetPluginConfig(
            core_lib::transfer::PluginConfig {},
        ))
        .warn_details("unable to send plugin config");

    let event_sender_2 = event_sender.clone();
    gio::spawn_blocking(move || {
        thread::sleep(Duration::from_millis(500));
        event_sender_2
            .send_blocking(TransferType::SetActive)
            .warn_details("unable to send");
    });

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
        active: false,
    };
    if let Some(windows) = &config.windows {
        let mut windows_data = WindowsGlobal::default();
        if let Some(overview) = &windows.overview {
            // let overview_data =
            //     windows_lib::overview::create_windows_overview_window(app, overview, windows)
            //         .context("failed to create overview window")?;
            let overview_data = todo!();
            let launcher_data = create_windows_overview_launcher_window(
                app,
                &overview.launcher,
                data_dir,
                &event_sender,
            )
            .context("failed to create launcher window")?;
            windows_data.overview = Some((overview_data, launcher_data, false));
        } else {
            debug!("Windows overview disabled");
        }
        if let Some(switch) = &windows.switch {
            // let switch_data = windows_lib::switch::create_windows_switch_window(
            //     app,
            //     switch,
            //     windows,
            //     event_sender,
            // )
            // .context("failed to create switch window")?;
            // windows_data.switch = Some(switch_data);
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

    provider_app.load_from_string(include_str!("default_styles.css"));
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
    config_file: Rc<PathBuf>,
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
        setup_restart_listener(&config_file, &css_path, &restart_sender);
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

fn setup_restart_listener(config_file: &Path, css_path: &Path, restart_tx: &Sender<&'static str>) {
    let tx = restart_tx.clone();
    if let Ok(watcher) = hyprshell_config_listener(config_file, move |mess| {
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
    let tx = restart_tx.clone();
    glib::spawn_future_local(async move {
        hyprland_config_listener(move |mess| {
            let _ = tx.send_blocking(mess);
        })
        .await;
    });
}
