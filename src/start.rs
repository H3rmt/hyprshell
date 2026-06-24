use crate::root::{Root, RootInit};
use crate::socket::socket_handler;
use crate::util;
use crate::util::check_new_version;
use crate::wm::configure_wm_initial;
use anyhow::Context;
use async_channel::Sender;
use core_lib::listener::{hyprshell_config_listener, hyprshell_css_listener};
use core_lib::transfer::ExternalTransferType;
use core_lib::{WarnWithDetails, notify, notify_resident, notify_warn};
use exec_lib::listener::{hyprland_config_listener, monitor_listener};
use relm4::RelmApp;
use relm4::adw::gtk::glib;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use std::{env, thread};
use tracing::{debug, info, trace};

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
                Duration::from_secs(3),
            );
            thread::sleep(Duration::from_secs(1));
            for info in messages {
                notify_resident(&info, Duration::from_secs(12));
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

    let (external_event_sender, external_event_receiver) = async_channel::unbounded();

    if env::var_os("HYPRSHELL_NO_LISTENERS").is_none() {
        register_event_restarter(
            config_file.clone(),
            css_path.clone(),
            external_event_sender.clone(),
        );
    }
    thread::spawn(move || {
        socket_handler(&external_event_sender);
    });
    configure_wm_initial();

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

    relm.run::<Root>(RootInit {
        external_event_receiver,
        data_dir,
        config_file,
        css_path,
        cache_dir,
    });

    Ok(())
}

pub fn register_event_restarter(
    config_file: Rc<PathBuf>,
    css_path: Rc<PathBuf>,
    event_sender: Sender<ExternalTransferType>,
) {
    let (restart_sender, restart_receiver) = async_channel::unbounded();

    // State to track the current debounce timer
    let debounce_delay = env::var("HYPRSHELL_RELOAD_DEBOUNCE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
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
            let timer_id =
                glib::timeout_add_local_once(Duration::from_millis(debounce_delay), move || {
                    trace!("Debounce timer expired, triggering restart ({cause})");

                    // Clear the timer reference since it's about to complete
                    *debounce_timer_clone.borrow_mut() = None;

                    // Send the restart event
                    let event_sender_inner = event_sender_clone.clone();
                    glib::spawn_future_local(async move {
                        info!("Restarting gui ({cause})");
                        event_sender_inner
                            .send(ExternalTransferType::Reload)
                            .await
                            .warn_details("unable to send restart");
                    });
                });

            // Store the timer ID so we can cancel it if needed
            *debounce_timer.borrow_mut() = Some(timer_id);
        }
    });

    // delay for 1.5 seconds to allow the config to be reloaded before listening for reload
    let delay = env::var("HYPRSHELL_RELOAD_DELAY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    // rustdoc of this function is wrong, waits for delay then runs func once
    glib::timeout_add_local_once(Duration::from_millis(delay), move || {
        setup_restart_listener(&config_file, &css_path, &restart_sender);
    });
}

fn setup_restart_listener(config_file: &Path, css_path: &Path, restart_tx: &Sender<&'static str>) {
    let tx = restart_tx.clone();
    let mut buffer = [0u8; 4096];
    if let Ok(mut watcher) = hyprshell_config_listener(config_file) {
        glib::spawn_future_local(async move {
            let events = watcher
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");
            for event in events {
                trace!("Received events: {event:?}");
                let _ = tx.send_blocking("config");
            }
        });
    }
    let tx = restart_tx.clone();
    let mut buffer2 = [0u8; 4096];
    if let Ok(mut watcher) = hyprshell_css_listener(css_path) {
        glib::spawn_future_local(async move {
            let events = watcher
                .read_events_blocking(&mut buffer2)
                .expect("Failed to read inotify events");
            for event in events {
                trace!("Received events: {event:?}");
                let _ = tx.send_blocking("config");
            }
        });
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
