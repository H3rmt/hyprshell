use crate::util::to_client_address;
use anyhow::Context;
use core_lib::{ClientId, Warn, WorkspaceAddress};
use hyprland::data::{Client, Monitors, Workspace, WorkspaceKind, Workspaces};
use hyprland::prelude::*;
use tracing::{debug, instrument, trace};

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
pub fn switch_workspace(workspace_address: WorkspaceAddress) -> anyhow::Result<()> {
    if Workspace::get_active().is_ok_and(|workspace| workspace.address == workspace_address) {
        trace!("Already on workspace {workspace_address}");
        return Ok(());
    }

    if Monitors::get()?
        .iter()
        .any(|monitor| monitor.special_workspace.address == workspace_address)
    {
        trace!("Special workspace is already active on a monitor: {workspace_address}");
        return Ok(());
    }

    deactivate_special_workspace_if_needed().warn();

    let workspace = Workspaces::get()?
        .into_iter()
        .find(|workspace| workspace.address == workspace_address)
        .context("workspace not found")?;

    if workspace.workspace_type == WorkspaceKind::Special {
        switch_special_workspace(&workspace_address).with_context(|| {
            format!("Failed to execute switch special workspace {workspace_address}")
        })?;
    } else {
        switch_normal_workspace(&workspace_address)
            .with_context(|| format!("Failed to execute switch workspace {workspace_address}"))?;
    }
    Ok(())
}

fn switch_special_workspace(workspace_address: &WorkspaceAddress) -> anyhow::Result<()> {
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Address(workspace_address.clone()),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

fn switch_normal_workspace(workspace_address: &WorkspaceAddress) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_address}");
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Address(workspace_address.clone()),
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
    let Some(active_client) = Client::get_active().context("active client failed")? else {
        return Ok(());
    };
    let workspace = active_client.workspace;

    trace!("current window workspace: {}", workspace.address);
    if workspace.workspace_type == WorkspaceKind::Special {
        debug!("current window is on a special workspace, deactivating it");
        let disp = hyprland::dispatch_new::Dispatch::ToggleSpecialWorkspace(
            special_workspace_name(&workspace.name).to_string(),
        );
        disp.apply().context("failed to execute dispatch")?;
    }
    Ok(())
}

fn special_workspace_name(address: &WorkspaceAddress) -> &str {
    address.strip_prefix("special:").unwrap_or(address)
}
