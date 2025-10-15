use crate::data::{SortConfig, collect_data};
use crate::global::WindowsSwitchData;
use crate::next::{
    get_curr_client_pos, get_curr_workspace_pos, select_next_active_client,
    select_next_active_workspace,
};
use crate::render_switch;
use anyhow::Context;
use core_lib::transfer::TransferType;
use exec_lib::switch::{switch_client, switch_workspace};
use exec_lib::to_client_address;
use tracing::debug_span;

pub fn refresh_switch(
    data: &mut WindowsSwitchData,
    transfer_type: &TransferType,
) -> anyhow::Result<()> {
    match transfer_type {
        TransferType::CloseSwitchItem => refresh_after_close_item(data),
        _ => Err(anyhow::anyhow!("Unexpected transfer type for refresh")),
    }
}

fn refresh_after_close_item(data: &mut WindowsSwitchData) -> anyhow::Result<()> {
    let _span = debug_span!("refresh_switch").entered();

    // Get the position of the current active item before refresh
    let current_pos = if data.config.switch_workspaces {
        let id = data.active.workspace;
        get_curr_workspace_pos(&data.hypr_data.workspaces, id)
    } else {
        data.active
            .client
            .and_then(|id| get_curr_client_pos(&data.hypr_data.clients, id))
    };

    // Recollect Hyprland data to get current state
    let (clients_data, active_prev) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: true,
    })
    .context("Failed to collect data")?;

    let prev_active = data.active.clone();

    // Determine what should be active after refresh
    let next_active = if data.config.switch_workspaces {
        select_next_active_workspace(
            data.active.workspace,
            current_pos,
            &clients_data,
            active_prev,
            data.config.items_per_row as usize,
        )
    } else {
        select_next_active_client(
            data.active.client,
            current_pos,
            &clients_data,
            active_prev,
            data.config.items_per_row as usize,
        )
    };

    // Handle case where client / workspace fails to be closed. This could happen
    // if the application interrupts its termination (like a browser asking to confirm)
    // before closing all tabs. In that case, we focus the client / workspace so
    // the user can immediately see any confirmation dialogue.
    if data.config.switch_workspaces {
        if prev_active.workspace == next_active.workspace {
            // workspace wasn't closed
            let _ = switch_workspace(prev_active.workspace);
        }
    } else if prev_active.client == next_active.client && prev_active.client.is_some() {
        // client wasn't closed
        let addr = to_client_address(prev_active.client.unwrap());
        let _ = switch_client(addr);
    }

    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    render_switch(data, clients_data, next_active, &remove_html)
}
