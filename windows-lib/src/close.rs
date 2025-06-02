use crate::WindowsGlobal;
use core_lib::{FindByFirst, IdOverride, Warn};
use exec_lib::switch::{switch_client, switch_workspace};
use exec_lib::{activate_submap, to_client_address};
use gtk::glib;
use gtk::prelude::*;
use tracing::{Level, debug, span, trace};

pub fn close_overview(global: &WindowsGlobal, ids: Option<Option<IdOverride>>) {
    let _span = span!(Level::TRACE, "close_overview").entered();

    activate_submap("reset").warn("Failed to reset submap");
    let mut data1 = global.data.borrow_mut();
    for (window, monitor_data) in &mut data1.monitor_list.iter_mut() {
        while let Some(child) = monitor_data.workspaces_flow.first_child() {
            monitor_data.workspaces_flow.remove(&child);
        }
        trace!("Hiding window (windows) {:?}", window.id());
        window.set_visible(false);
    }

    if let Some(ids) = ids {
        let ids = match ids {
            None => data1
                .active
                .client
                .map(IdOverride::ClientId)
                .unwrap_or_else(|| IdOverride::WorkspaceID(data1.active.workspace)),
            Some(IdOverride::ClientId(client_id)) => IdOverride::ClientId(client_id),
            Some(IdOverride::WorkspaceID(workspace_id)) => IdOverride::WorkspaceID(workspace_id),
        };
        match ids {
            IdOverride::ClientId(client_id) => {
                debug!(
                    "Switching to client {}",
                    data1
                        .hypr_data
                        .clients
                        .find_by_first(&client_id)
                        .map(|c| c.title.clone())
                        .unwrap_or_else(|| "<Unknown>".to_string())
                );
                // we need to do this because the window might still be visible and have KeyboardMode::Exclusive
                glib::idle_add_local(move || {
                    switch_client(to_client_address(client_id))
                        .warn(&format!("Failed to execute with id {client_id:?}"));
                    glib::ControlFlow::Break
                });
            }
            IdOverride::WorkspaceID(workspace_id) => {
                debug!(
                    "Switching to workspace {}",
                    data1
                        .hypr_data
                        .workspaces
                        .find_by_first(&workspace_id)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "<Unknown>".to_string())
                );
                // we need to do this because the window might still be visible and have KeyboardMode::Exclusive
                glib::idle_add_local(move || {
                    switch_workspace(workspace_id).warn(&format!(
                        "Failed to execute switch workspace with id {workspace_id:?}"
                    ));
                    glib::ControlFlow::Break
                });
            }
        }
    }
}
