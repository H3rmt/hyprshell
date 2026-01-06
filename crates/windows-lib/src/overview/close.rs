use crate::global::WindowsOverviewData;
use core_lib::transfer::WindowsOverride;
use core_lib::{FindByFirst, WarnWithDetails};
use exec_lib::switch::{switch_client, switch_workspace};
use exec_lib::{reset_no_follow_mouse, to_client_address};
use relm4::adw::gtk::glib;
use relm4::adw::gtk::prelude::*;
use tracing::{debug, debug_span, trace};

#[must_use]
pub fn overview_already_hidden(data: &WindowsOverviewData) -> bool {
    !data.window_list.iter().any(|w| w.0.get_visible())
}

pub fn close_overview(data: &mut WindowsOverviewData, ids: Option<Option<WindowsOverride>>) {
    let _span = debug_span!("close_overview").entered();
    reset_no_follow_mouse().warn_details("Failed to reset follow mouse");

    for (window, monitor_data) in &mut data.window_list.iter_mut() {
        while let Some(child) = monitor_data.workspaces_flow.first_child() {
            monitor_data.workspaces_flow.remove(&child);
        }
        trace!("Hiding window (windows) {:?}", window.id());
        window.set_visible(false);
    }

    if let Some(ids) = ids {
        let ids = match ids {
            None => data.active.client.map_or_else(
                || WindowsOverride::WorkspaceID(data.active.workspace),
                WindowsOverride::ClientId,
            ),
            Some(WindowsOverride::ClientId(client_id)) => WindowsOverride::ClientId(client_id),
            Some(WindowsOverride::WorkspaceID(workspace_id)) => {
                WindowsOverride::WorkspaceID(workspace_id)
            }
        };
        match ids {
            WindowsOverride::ClientId(client_id) => {
                debug!(
                    "Switching to client {}",
                    data.hypr_data
                        .clients
                        .find_by_first(&client_id)
                        .map_or_else(|| "<Unknown>".to_string(), |c| c.title.clone())
                );
                glib::idle_add_local(move || {
                    switch_client(to_client_address(client_id))
                        .warn_details(&format!("Failed to execute with id {client_id:?}"));
                    glib::ControlFlow::Break
                });
            }
            WindowsOverride::WorkspaceID(workspace_id) => {
                debug!(
                    "Switching to workspace {}",
                    data.hypr_data
                        .workspaces
                        .find_by_first(&workspace_id)
                        .map_or_else(|| "<Unknown>".to_string(), |c| c.name.clone())
                );
                glib::idle_add_local(move || {
                    switch_workspace(workspace_id).warn_details(&format!(
                        "Failed to execute switch workspace with id {workspace_id:?}"
                    ));
                    glib::ControlFlow::Break
                });
            }
        }
    }
}
