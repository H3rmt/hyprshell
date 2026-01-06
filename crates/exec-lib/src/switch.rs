use anyhow::Context;
use core_lib::Warn;
use hyprland::data::{Client, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{
    Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::{Address, WorkspaceId};
use tracing::{debug, trace};

pub fn switch_client(address: Address) -> anyhow::Result<()> {
    debug!("execute switch to client: {address}");
    deactivate_special_workspace_if_needed().warn();
    Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(
        address,
    )))?;
    Dispatch::call(DispatchType::BringActiveToTop)?;
    Ok(())
}

pub fn switch_client_by_initial_class(class: &str) -> anyhow::Result<()> {
    debug!("execute switch to client: {class} by initial_class");
    deactivate_special_workspace_if_needed().warn();
    Dispatch::call(DispatchType::FocusWindow(
        WindowIdentifier::ClassRegularExpression(&format!(
            "initialclass:{}",
            class.to_ascii_lowercase()
        )),
    ))?;
    Dispatch::call(DispatchType::BringActiveToTop)?;
    Ok(())
}

pub fn kill_client(address: Address) -> anyhow::Result<()> {
    Dispatch::call(DispatchType::CloseWindow(WindowIdentifier::Address(
        address,
    )))?;
    Ok(())
}

pub fn switch_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    deactivate_special_workspace_if_needed().warn();

    // check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
    let current_workspace = Workspace::get_active();
    if let Ok(workspace) = current_workspace
        && workspace_id == workspace.id
    {
        trace!("Already on workspace {}", workspace_id);
        return Ok(());
    }

    if workspace_id < 0 {
        switch_special_workspace(workspace_id).with_context(|| {
            format!("Failed to execute switch special workspace with id {workspace_id}")
        })?;
    } else {
        switch_normal_workspace(workspace_id).with_context(|| {
            format!("Failed to execute switch workspace with id {workspace_id}")
        })?;
    }
    Ok(())
}

fn switch_special_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    let special = Monitors::get()?
        .into_iter()
        .find(|m| m.special_workspace.id == workspace_id);
    if let Some(special) = special {
        trace!("Special workspace already toggled: {special:?}");
        return Ok(());
    }
    let ws = Workspaces::get()?
        .into_iter()
        .find(|w| w.id == workspace_id)
        .context("workspace not found")?;
    Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(
        ws.name.trim_start_matches("special:").to_string(),
    )))
    .context("failed to execute toggle special workspace")
}

fn switch_normal_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
        workspace_id,
    )))?;
    Ok(())
}

/// always run when changing client or workspace
///
/// if client on special workspace is opened the workspace is activated
fn deactivate_special_workspace_if_needed() -> anyhow::Result<()> {
    let active_ws = Workspace::get_active()
        .map(|w| w.name)
        .context("active workspace failed")?;
    let active_ws = Client::get_active()
        .context("active client failed")?
        .map_or(active_ws, |a| a.workspace.name);
    trace!("current workspace: {active_ws}");
    if active_ws.starts_with("special:") {
        debug!("current client is on special workspace, deactivating special workspace");
        // current client is on special workspace
        Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(
            active_ws.trim_start_matches("special:").to_string(),
        )))?;
    }
    Ok(())
}
