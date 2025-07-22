use crate::start::Globals;
use async_channel::{Receiver, Sender};
use core_lib::WarnWithDetails;
use core_lib::transfer::{
    CloseOverviewConfig, OpenSwitch, SwitchOverviewConfig, SwitchSwitchConfig, TransferType,
};
use gtk::prelude::EntryExt;
use tracing::{debug, trace, warn};

pub async fn event_handler(
    mut globals: Globals,
    event_receiver: Receiver<TransferType>,
    event_sender: Sender<TransferType>,
) {
    loop {
        if let Ok(transfer) = event_receiver.recv().await {
            let close_socket = matches!(transfer, TransferType::Restart);
            trace!("handling event: {transfer:?}");
            match transfer {
                TransferType::OpenOverview => open_overview(&mut globals, event_sender.clone()),
                TransferType::OpenSwitch(config) => open_switch(&mut globals, config),
                TransferType::SwitchOverview(config) => switch_overview(&mut globals, config),
                TransferType::SwitchSwitch(config) => switch_switch(&mut globals, config),
                TransferType::Exit => exit(&mut globals),
                TransferType::Type(text) => r#type(&mut globals, text, event_sender.clone()),
                TransferType::CloseOverview(config) => close_overview(&mut globals, config),
                TransferType::CloseSwitch => close_switch(&mut globals),
                TransferType::Restart => restart(&globals),
            }
            if close_socket {
                return;
            }
        }
    }
}

fn r#type(global: &mut Globals, text: String, event_sender: Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let Some((_overview, launcher)) = &mut windows.overview {
            launcher_lib::update_launcher(launcher, text, event_sender)
        }
    }
}

fn open_overview(global: &mut Globals, event_sender: Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher)) = &mut windows.overview {
            if !windows_lib::overview_already_open(overview)
                && !&windows
                    .switch
                    .as_ref()
                    .map(windows_lib::switch_already_open)
                    .unwrap_or(false)
            {
                windows_lib::open_overview(overview, event_sender)
                    .warn_details("Failed to open overview window");
                launcher_lib::open_launcher(launcher)
            } else {
                debug!("Overview or Switch already open");
            }
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    };
}

fn open_switch(global: &mut Globals, config: OpenSwitch) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if !windows_lib::switch_already_open(switch)
                && !&windows
                    .overview
                    .as_ref()
                    .map(|(o, _)| windows_lib::overview_already_open(o))
                    .unwrap_or(false)
            {
                windows_lib::open_switch(switch, config)
                    .warn_details("Failed to open switch window");
            } else {
                debug!("Switch or Overview already open");
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn switch_switch(global: &mut Globals, config: SwitchSwitchConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            windows_lib::update_switch(switch, config)
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn switch_overview(global: &mut Globals, config: SwitchOverviewConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher)) = &mut windows.overview {
            // don't switch selected window if launcher is active
            let launch = launcher.entry.text_length() > 0;
            if !launch {
                windows_lib::update_overview(overview, config);
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn exit(global: &mut Globals) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher)) = &mut windows.overview {
            windows_lib::close_overview(overview, None);
            launcher_lib::close_launcher_by_char(launcher, None); // this will never open a program and need the default terminal
        };
        if let Some(switch) = &mut windows.switch {
            windows_lib::close_switch(switch, false);
        };
    }
}

fn close_overview(global: &mut Globals, config: CloseOverviewConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some((overview, launcher)) = &mut windows.overview {
            if windows_lib::overview_already_hidden(overview) {
                debug!("Overview is already closed");
                return;
            }
            match config {
                // return (focus active)
                CloseOverviewConfig::None => {
                    let launcher_empty = launcher.entry.text_length() == 0;
                    let other_active = overview.active != overview.initial_active;
                    let launcher_no_items = launcher.sorted_matches.is_empty();
                    if launcher_empty && other_active {
                        // close overview, kill launcher
                        windows_lib::close_overview(overview, Some(None));
                        launcher_lib::close_launcher_by_char(launcher, None);
                    } else if launcher_no_items {
                        debug!("Launcher is empty, not closing");
                    } else {
                        // kill overview, close launcher
                        windows_lib::close_overview(overview, None);
                        launcher_lib::close_launcher_by_char(launcher, Some('0'));
                    };
                }
                // clicked on launcher item
                CloseOverviewConfig::LauncherClick(iden) => {
                    windows_lib::close_overview(overview, None);
                    launcher_lib::close_launcher_by_iden(launcher, &iden);
                }
                // typed a character in launcher
                CloseOverviewConfig::LauncherPress(iden) => {
                    windows_lib::close_overview(overview, None);
                    launcher_lib::close_launcher_by_char(launcher, Some(iden));
                }
                // clicked on window
                CloseOverviewConfig::Windows(iden) => {
                    windows_lib::close_overview(overview, Some(Some(iden)));
                    launcher_lib::close_launcher_by_char(launcher, None);
                }
            }
        }
    }
}

fn close_switch(global: &mut Globals) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if windows_lib::switch_already_hidden(switch) {
                debug!("Switch is already closed");
                return;
            }
            windows_lib::close_switch(switch, true);
        }
    }
}

fn restart(global: &Globals) {
    if let Some(windows) = &global.windows {
        if let Some((overview, launcher)) = &windows.overview {
            windows_lib::stop_overview(overview);
            launcher_lib::stop_launcher(launcher);
        };
        if let Some(switch) = &windows.switch {
            windows_lib::stop_switch(switch);
        };
    }
}
