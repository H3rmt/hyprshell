use crate::util::to_client_address;
use anyhow::Context;
use core_lib::{ClientId, Warn};
use hyprland::data::{Client, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{
    Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::WorkspaceId;
use tracing::{debug, instrument, trace, warn};

#[instrument(level = "debug", ret(level = "trace"))]
pub fn switch_client(address: ClientId) -> anyhow::Result<()> {
    match switch_client_lua(address) {
        Err(e) => {
            warn!("Failed to switch to client: {}, trying legacy syntax", e);
            switch_client_legacy(address)
        }
        Ok(r) => Ok(r),
    }
}

fn switch_client_legacy(address: ClientId) -> anyhow::Result<()> {
    debug!("execute switch to client: {address}");
    deactivate_special_workspace_if_needed().warn();
    Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(
        to_client_address(address),
    )))
    .context("failed to execute dispatch")?;

    Dispatch::call(DispatchType::BringActiveToTop).context("failed to execute dispatch2")?;
    Ok(())
}

fn switch_client_lua(address: ClientId) -> anyhow::Result<()> {
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
    match switch_client_by_initial_class_lua(class) {
        Err(e) => {
            warn!("Failed to switch to client: {}, trying legacy syntax", e);
            switch_client_by_initial_class_legacy(class)
        }
        Ok(r) => Ok(r),
    }
}

fn switch_client_by_initial_class_lua(class: &str) -> anyhow::Result<()> {
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

fn switch_client_by_initial_class_legacy(class: &str) -> anyhow::Result<()> {
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
    match switch_special_workspace_lua(workspace_id) {
        Err(e) => {
            warn!(
                "Failed to switch special workspace: {}, trying legacy syntax",
                e
            );
            switch_special_workspace_legacy(workspace_id)
        }
        Ok(r) => Ok(r),
    }
}

fn switch_special_workspace_legacy(workspace_id: WorkspaceId) -> anyhow::Result<()> {
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
    .context("failed to execute dispatch")?;
    Ok(())
}

fn switch_special_workspace_lua(workspace_id: WorkspaceId) -> anyhow::Result<()> {
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
    match switch_normal_workspace_lua(workspace_id) {
        Err(e) => {
            warn!("Failed to switch workspace: {}, trying legacy syntax", e);
            switch_normal_workspace_legacy(workspace_id)
        }
        Ok(r) => Ok(r),
    }
}

fn switch_normal_workspace_lua(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Id(workspace_id),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

fn switch_normal_workspace_legacy(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
        workspace_id,
    )))
    .context("failed to execute dispatch")?;
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
        match deactivate_special_workspace_if_needed_lua(&active_ws) {
            Err(e) => {
                warn!(
                    "Failed to deactivate special workspace: {}, trying legacy syntax",
                    e
                );
                deactivate_special_workspace_if_needed_legacy(&active_ws)
            }
            Ok(r) => Ok(r),
        }?;
    }
    Ok(())
}

fn deactivate_special_workspace_if_needed_lua(name: &str) -> anyhow::Result<()> {
    debug!("execute switch to workspace {name}");
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Special(Some(
            name.trim_start_matches("special:").to_string(),
        )),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

fn deactivate_special_workspace_if_needed_legacy(name: &str) -> anyhow::Result<()> {
    debug!("execute switch to workspace {name}");
    Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(
        name.trim_start_matches("special:").to_string(),
    )))?;
    Ok(())
}
