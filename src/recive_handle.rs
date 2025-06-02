use crate::receive::Globals;
use core_lib::transfer::{CloseConfig, OpenOverview, OpenSwitch, SwitchConfig, WindowsOverride};
use core_lib::{IdOverride, Warn, collect_desktop_files};
use gtk::prelude::EntryExt;
use tracing::{debug, warn};

macro_rules! if_some {
    ($global:expr, $func:path $(, $args:expr)*) => {
        if let Some(_gl) = &$global {
            $func(_gl $(, $args)*)
        }
    };
}
pub fn open_overview(global: &Globals, config: OpenOverview) {
    if let Some(global) = &global.window {
        windows_lib::open_overview(global, config).warn("Failed to open overview window");
    } else {
        warn!("Window overview not active");
    };
    if_some!(&global.launcher, launcher_lib::open_launcher);
}

pub fn open_switch(global: &Globals, config: OpenSwitch) {
    if let Some(global) = &global.window {
        windows_lib::open_switch(global, config).warn("Failed to open switch window");
    } else {
        warn!("Window switch not active");
    }
}

pub fn switch(global: &Globals, config: SwitchConfig) {
    if global.launcher.is_some() {
        let launch = global
            .launcher
            .as_ref()
            .and_then(|l| {
                l.data.as_ref().map(|d| {
                    let b = d.borrow();
                    b.entry.text_length() > 0
                })
            })
            .unwrap_or(false);
        // don't switch selected window if launcher is active
        if !launch {
            if_some!(&global.window, windows_lib::update_overview, config)
        }
    } else {
        if_some!(&global.window, windows_lib::update_overview, config)
    }
}
pub fn exit(global: &Globals) {
    if_some!(global.launcher, launcher_lib::close_launcher_press, None);
    if_some!(global.window, windows_lib::close_overview, None);
    reload_data();
}

pub fn r#type(global: &Globals, text: String) {
    if_some!(global.launcher, launcher_lib::update_launcher, text);
}

pub fn close(global: &Globals, config: CloseConfig) {
    match config {
        CloseConfig::None => {
            let (launcher_active, launcher_empty) = global
                .launcher
                .as_ref()
                .and_then(|l| {
                    l.data.as_ref().map(|d| {
                        let b = d.borrow();
                        (b.entry.text_length() > 0, b.sorted_matches.is_empty())
                    })
                })
                .unwrap_or((false, false));

            if launcher_active {
                if !launcher_empty {
                    // kill overview, close launcher
                    if_some!(
                        global.launcher,
                        launcher_lib::close_launcher_press,
                        Some('0')
                    );
                    if_some!(global.window, windows_lib::close_overview, None);
                } else {
                    debug!("Launcher is empty, not closing");
                }
            } else {
                // close overview, kill launcher
                if_some!(global.launcher, launcher_lib::close_launcher_press, None);
                if_some!(global.window, windows_lib::close_overview, Some(None));
            };
        }
        CloseConfig::LauncherClick(iden) => {
            // kill overview, close launcher
            if_some!(global.launcher, launcher_lib::close_launcher_click, iden);
            if_some!(global.window, windows_lib::close_overview, None);
        }
        CloseConfig::LauncherPress(char) => {
            // kill overview, close launcher
            if_some!(
                global.launcher,
                launcher_lib::close_launcher_press,
                Some(char)
            );
            if_some!(global.window, windows_lib::close_overview, None);
        }
        CloseConfig::Windows(WindowsOverride::ClientId(client_id)) => {
            // close overview, kill launcher
            if_some!(global.launcher, launcher_lib::close_launcher_press, None);
            if_some!(
                global.window,
                windows_lib::close_overview,
                Some(Some(IdOverride::ClientId(client_id)))
            );
        }
        CloseConfig::Windows(WindowsOverride::WorkspaceID(workspace_id)) => {
            // close overview, kill launcher
            if_some!(global.launcher, launcher_lib::close_launcher_press, None);
            if_some!(
                global.window,
                windows_lib::close_overview,
                Some(Some(IdOverride::WorkspaceID(workspace_id)))
            );
        }
    }
    reload_data();
}

pub fn restart(global: &Globals) {
    if_some!(&global.window, windows_lib::stop_overview);
    if_some!(&global.launcher, launcher_lib::stop_launcher);
}

pub fn reload_data() {
    // reload the desktop maps for the launcher and overview
    let desktop_files = collect_desktop_files();
    windows_lib::reload_desktop_map(&desktop_files);
    launcher_lib::reload_applications_desktop_map(&desktop_files);
    launcher_lib::reload_search_default_browser(&desktop_files);
}
