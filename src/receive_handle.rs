use crate::start::Globals;
use crate::util;
use async_channel::{Receiver, Sender};
use core_lib::WarnWithDetails;
use core_lib::transfer::{
    CloseOverviewConfig, Direction, OpenSwitch, SwitchOverviewConfig, SwitchSwitchConfig,
    TransferType,
};
use relm4::adw::gtk::prelude::ApplicationExt;
use relm4::adw::gtk::{gio, glib};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, instrument, trace, warn};

#[allow(clippy::future_not_send)]
#[instrument(level = "trace", skip_all)]
pub async fn event_handler(
    mut globals: Globals,
    event_receiver: Receiver<TransferType>,
    event_sender: Sender<TransferType>,
) {
    loop {
        if let Ok(transfer) = event_receiver.recv().await {
            let close_socket = matches!(transfer, TransferType::Restart);
            trace!("handling event: {transfer:?}");

            if !globals.active {
                match transfer {
                    TransferType::SetActive
                    | TransferType::Restart
                    | TransferType::SetPluginConfig(_) => {}
                    _ => {
                        debug!("Application is not active, ignoring event");
                        continue;
                    }
                }
            }
            match transfer {
                TransferType::OpenOverview => open_overview(&mut globals, &event_sender),
                TransferType::OpenSwitch(config) => open_switch(&mut globals, &config),
                TransferType::SwitchOverview(config) => switch_overview(&mut globals, &config),
                TransferType::SwitchSwitch(config) => switch_switch(&mut globals, &config),
                TransferType::CloseClientSwitch => close_client_switch(&mut globals, &event_sender),
                TransferType::CloseClientOverview => {
                    close_client_overview(&mut globals, &event_sender);
                }
                TransferType::Type(text) => r#type(&mut globals, &text, &event_sender),
                TransferType::CloseOverview(config) => close_overview(&mut globals, config),
                TransferType::CloseSwitch => close_switch(&mut globals),
                TransferType::CloseAll => close_all(&mut globals),
                TransferType::Restart => restart(&mut globals),
                TransferType::SetActive => {
                    globals.active = true;
                    info!("Application is now active");
                }
                TransferType::SetPluginConfig(_) => {}
                TransferType::GetConfigWatch => {}
            }
            if close_socket {
                return;
            }
        }
    }
}

fn r#type(global: &mut Globals, text: &str, event_sender: &Sender<TransferType>) {
    if let Some(windows) = &mut global.windows
        && let Some((_, launcher, launcher_active)) = &mut windows.overview
    {
        *launcher_active = !text.is_empty();
        launcher_lib::update_launcher(launcher, text, event_sender);
    }
}

fn open_overview(global: &mut Globals, event_sender: &Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher, launcher_active)) = &mut windows.overview {
            *launcher_active = false;
            if !windows_lib::overview::overview_already_open(overview)
                && !&windows
                    .switch
                    .as_ref()
                    .is_some_and(windows_lib::switch::switch_already_open)
            {
                trace!("Opening overview");
                windows_lib::overview::open_overview(overview, event_sender)
                    .warn_details("Failed to open overview window");
                trace!("Opening launcher");
                launcher_lib::open_launcher(launcher);
                trace!("Updating Launcher");
                launcher_lib::update_launcher(launcher, "", event_sender);

                // update desktop data in background
                trace!("Reloading desktop data");
                gio::spawn_blocking(util::reload_desktop_data);
            } else {
                debug!("Overview or Switch already open, closing");
                windows_lib::overview::close_overview(overview, None);
                launcher_lib::close_launcher_by_char(launcher, None); // this will never open a program and need the default terminal
            }
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn open_switch(global: &mut Globals, config: &OpenSwitch) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if !windows_lib::switch::switch_already_open(switch)
                && !&windows
                    .overview
                    .as_ref()
                    .is_some_and(|(o, _, _)| windows_lib::overview::overview_already_open(o))
            {
                windows_lib::switch::open_switch(switch, config)
                    .warn_details("Failed to open switch window");
            } else {
                debug!("Switch or Overview already open, converting to SwitchSwitch");
                windows_lib::switch::switch_to_next(
                    switch,
                    &SwitchSwitchConfig {
                        direction: if config.reverse {
                            Direction::Left
                        } else {
                            Direction::Right
                        },
                    },
                );
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn switch_switch(global: &mut Globals, config: &SwitchSwitchConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            windows_lib::switch::switch_to_next(switch, config);
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn close_client_switch(global: &mut Globals, _event_sender: &Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            let success = windows_lib::switch::close_item(switch)
                .warn_details("Failed to close switch item")
                .unwrap_or(false);
            if success {
                windows_lib::switch::switch_to_next(
                    switch,
                    &SwitchSwitchConfig {
                        direction: Direction::Right,
                    },
                );
                windows_lib::switch::update_data(switch)
                    .warn_details("unable to update switch data");
            } else {
                windows_lib::switch::close_switch(switch, true);
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn close_client_overview(global: &mut Globals, event_sender: &Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, _, _)) = &mut windows.overview {
            let success = windows_lib::overview::close_client(overview)
                .warn_details("Failed to close switch item")
                .unwrap_or(false);
            if success {
                windows_lib::overview::switch_to_next(
                    overview,
                    &SwitchOverviewConfig {
                        direction: Direction::Right,
                        workspace: false,
                    },
                );
                windows_lib::overview::update_data(overview, event_sender)
                    .warn_details("unable to update switch data");
            } else {
                windows_lib::overview::close_overview(overview, Some(None));
            }
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn switch_overview(global: &mut Globals, config: &SwitchOverviewConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, _, launcher_active)) = &mut windows.overview {
            if !*launcher_active {
                windows_lib::overview::switch_to_next(overview, config);
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn close_all(global: &mut Globals) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher, _)) = &mut windows.overview {
            windows_lib::overview::close_overview(overview, None);
            launcher_lib::close_launcher_by_char(launcher, None); // this will never open a program and need the default terminal
        }
        if let Some(switch) = &mut windows.switch {
            windows_lib::switch::close_switch(switch, false);
        }
    }
}

fn close_overview(global: &mut Globals, config: CloseOverviewConfig) {
    if let Some(windows) = &mut global.windows
        && let Some((overview, launcher, launcher_active)) = &mut windows.overview
    {
        if windows_lib::overview::overview_already_hidden(overview) {
            debug!("Overview is already closed");
            return;
        }
        match config {
            // return (focus active)
            CloseOverviewConfig::None => {
                if *launcher_active {
                    if launcher.sorted_matches.is_empty() {
                        debug!("Launcher is empty, not closing");
                    } else {
                        // kill overview, close launcher
                        windows_lib::overview::close_overview(overview, None);
                        launcher_lib::close_launcher_by_char(launcher, Some('0'));
                    }
                } else {
                    // close overview, kill launcher
                    windows_lib::overview::close_overview(overview, Some(None));
                    launcher_lib::close_launcher_by_char(launcher, None);
                }
            }
            // clicked on launcher item
            CloseOverviewConfig::LauncherClick(iden) => {
                windows_lib::overview::close_overview(overview, None);
                launcher_lib::close_launcher_by_iden(launcher, &iden);
            }
            // typed a character in launcher
            CloseOverviewConfig::LauncherPress(iden) => {
                windows_lib::overview::close_overview(overview, None);
                launcher_lib::close_launcher_by_char(launcher, Some(iden));
            }
            // clicked on window
            CloseOverviewConfig::Windows(iden) => {
                windows_lib::overview::close_overview(overview, Some(Some(iden)));
                launcher_lib::close_launcher_by_char(launcher, None);
            }
        }
    }
}

fn close_switch(global: &mut Globals) {
    if let Some(windows) = &mut global.windows
        && let Some(switch) = &mut windows.switch
    {
        if windows_lib::switch::switch_already_hidden(switch) {
            debug!("Switch is already closed");
            return;
        }
        windows_lib::switch::close_switch(switch, true);
    }
}

fn restart(global: &mut Globals) {
    global.active = false;
    if let Some(windows) = &global.windows {
        if let Some((overview, launcher, _)) = &windows.overview {
            windows_lib::overview::stop_overview(overview);
            launcher_lib::stop_launcher(launcher);
        }
        if let Some(switch) = &windows.switch {
            // windows_lib::switch::stop_switch(switch);
        }
    }
    let app = global.app.clone();
    thread::sleep(Duration::from_millis(500));
    glib::idle_add_local_once(move || {
        app.quit();
        debug!("application closed");
    });
}
