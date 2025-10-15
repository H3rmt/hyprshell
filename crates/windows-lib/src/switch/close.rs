use crate::global::WindowsSwitchData;
use adw::gtk::glib;
use adw::gtk::prelude::*;
use async_channel::Sender;
use core_lib::transfer::TransferType;
use core_lib::{FindByFirst, WarnWithDetails};
use exec_lib::switch::{kill_client, switch_client, switch_workspace};
use exec_lib::{reset_no_follow_mouse, to_client_address};
use std::time::Duration;
use tracing::{debug, debug_span, trace};

#[must_use]
pub fn switch_already_hidden(data: &WindowsSwitchData) -> bool {
    !data.window.is_visible()
}

pub fn close_switch(data: &mut WindowsSwitchData, switch: bool) {
    let _span = debug_span!("close_switch").entered();

    reset_no_follow_mouse().warn_details("Failed to reset follow mouse");
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
                    .map_or_else(|| "<Unknown>".to_string(), |c| c.title.clone())
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
                    .map_or_else(|| "<Unknown>".to_string(), |c| c.name.clone())
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

pub fn close_switch_item(data: &WindowsSwitchData, event_sender: &Sender<TransferType>) {
    if data.config.switch_workspaces {
        kill_switch_workspace(data);
    } else {
        kill_switch_client(data);
    }

    let sender = event_sender.clone();
    glib::timeout_add_local(Duration::from_millis(50), move || {
        sender
            .try_send(TransferType::RefreshSwitch(Box::new(
                TransferType::CloseSwitchItem,
            )))
            .warn_details("Failed to send RefreshSwitch event");
        glib::ControlFlow::Break
    });
}

fn kill_switch_client(data: &WindowsSwitchData) {
    if let Some(id) = data.active.client {
        let addr = to_client_address(id);
        let _ = kill_client(addr);
    }
}

fn kill_switch_workspace(data: &WindowsSwitchData) {
    let workspace_id = data.active.workspace;
    debug!(
        "Killing all clients in workspace {}",
        data.hypr_data
            .workspaces
            .find_by_first(&workspace_id)
            .map_or_else(|| workspace_id.to_string(), |w| w.name.clone())
    );

    let clients_to_kill: Vec<_> = data
        .hypr_data
        .clients
        .iter()
        .filter(|(_, client)| client.workspace == workspace_id && client.enabled)
        .map(|(id, _)| *id)
        .collect();

    for client_id in clients_to_kill {
        let addr = to_client_address(client_id);
        let _ = kill_client(addr);
    }
}
