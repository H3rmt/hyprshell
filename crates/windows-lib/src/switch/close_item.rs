use crate::global::WindowsSwitchData;
use adw::gtk::glib;
use anyhow::{Context, Ok, anyhow};
use async_channel::Sender;
use core_lib::transfer::TransferType;
use core_lib::{FindByFirst, WarnWithDetails};
use exec_lib::switch::kill_client;
use exec_lib::to_client_address;
use std::time::Duration;
use tracing::debug;

pub fn close_switch_item(
    data: &WindowsSwitchData,
    event_sender: &Sender<TransferType>,
) -> anyhow::Result<()> {
    if data.config.switch_workspaces {
        kill_switch_workspace(data)?;
    } else {
        kill_switch_client(data)?;
    }

    let sender = event_sender.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        sender
            .try_send(TransferType::RefreshSwitch(Box::new(
                TransferType::CloseSwitchItem,
            )))
            .warn_details("Failed to send RefreshSwitch event");
        glib::ControlFlow::Break
    });

    Ok(())
}

fn kill_switch_client(data: &WindowsSwitchData) -> anyhow::Result<()> {
    if let Some(id) = data.active.client {
        let addr = to_client_address(id);
        return kill_client(addr).context("failed to kill active client");
    }
    Err(anyhow!("no active client"))
}

fn kill_switch_workspace(data: &WindowsSwitchData) -> anyhow::Result<()> {
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
        kill_client(addr).context("failed to kill client while killing workspace")?;
    }

    Ok(())
}
