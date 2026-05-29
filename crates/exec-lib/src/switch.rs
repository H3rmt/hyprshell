use crate::util::to_client_address;
use anyhow::Context;
use core_lib::{ClientId, Warn};
use hyprland::data::{Client, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::prelude::*;
use hyprland::shared::WorkspaceId;
use tracing::{debug, instrument, trace};

#[instrument(level = "debug", ret(level = "trace"))]
pub fn switch_client(address: ClientId) -> anyhow::Result<()> {
    debug!("execute switch to client: {address}");
    deactivate_special_workspace_if_needed().warn();
    let disp = hyprland::dispatch_new::Dispatch::FocusWindow(
        hyprland::dispatch_new::WindowIdentifier::Address(to_client_address(address)),
    );
    disp.apply().context("failed to execute dispatch")?;
    let disp2 = hyprland::dispatch_new::Dispatch::WindowAlterZ(
        hyprland::dispatch_new::ZOption::Top,
        Some(hyprland::dispatch_new::WindowIdentifier::Address(
            to_client_address(address),
        )),
    );
    disp2.apply().context("failed to execute dispatch2")?;
    Ok(())
}

#[instrument(level = "debug", ret(level = "trace"))]
pub fn switch_client_by_initial_class(class: &str) -> anyhow::Result<()> {
    debug!("execute switch to client: {class} by initial_class");
    deactivate_special_workspace_if_needed().warn();

    let disp = hyprland::dispatch_new::Dispatch::FocusWindow(
        hyprland::dispatch_new::WindowIdentifier::InitialClassRegularExpression(
            class.to_ascii_lowercase(),
        ),
    );
    disp.apply().context("failed to execute dispatch")?;
    let disp2 = hyprland::dispatch_new::Dispatch::WindowAlterZ(
        hyprland::dispatch_new::ZOption::Top,
        Some(
            hyprland::dispatch_new::WindowIdentifier::InitialClassRegularExpression(
                class.to_ascii_lowercase(),
            ),
        ),
    );
    disp2.apply().context("failed to execute dispatch2")?;
    Ok(())
}

#[instrument(level = "debug", ret(level = "trace"))]
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

#[instrument(level = "debug", ret(level = "trace"))]
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

    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Special(Some(
            ws.name.trim_start_matches("special:").to_string(),
        )),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

#[instrument(level = "debug", ret(level = "trace"))]
fn switch_normal_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Id(workspace_id),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

/// always run when changing client or workspace
///
/// if client on special workspace is opened the workspace is activated
#[instrument(level = "debug", ret(level = "trace"))]
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
