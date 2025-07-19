use crate::global::WindowsSwitchData;
use core_lib::{FindByFirst, WarnWithDetails};
use exec_lib::switch::{switch_client, switch_workspace};
use exec_lib::{reset_remain_focused, to_client_address};
use gtk::glib;
use gtk::prelude::*;
use tracing::{Level, debug, span, trace};

pub fn switch_already_hidden(data: &WindowsSwitchData) -> bool {
    !data.window.is_visible()
}

pub fn close_switch(data: &mut WindowsSwitchData, switch: bool) {
    let _span = span!(Level::TRACE, "close_switch").entered();

    reset_remain_focused().warn_details("Failed to reset follow mouse");
    while let Some(child) = data.main_flow.first_child() {
        data.main_flow.remove(&child);
    }
    trace!("Hiding window (windows) {:?}", data.window.id());
    data.window.set_visible(false);

    if switch {
        if let Some(id) = data.active.client {
            debug!(
                "Switching to client {}",
                data.hypr_data
                    .clients
                    .find_by_first(&id)
                    .map(|c| c.title.clone())
                    .unwrap_or_else(|| "<Unknown>".to_string())
            );
            // we need to do this because the window might still be visible and have KeyboardMode::Exclusive
            glib::idle_add_local(move || {
                switch_client(to_client_address(id))
                    .warn_details(&format!("Failed to execute with id {id:?}"));
                glib::ControlFlow::Break
            });
        } else {
            let id = data.active.workspace;
            debug!(
                "Switching to workspace {}",
                data.hypr_data
                    .workspaces
                    .find_by_first(&id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "<Unknown>".to_string())
            );
            glib::idle_add_local(move || {
                switch_workspace(id).warn_details(&format!(
                    "Failed to execute switch workspace with id {id:?}"
                ));
                glib::ControlFlow::Break
            });
        }
    }
}
