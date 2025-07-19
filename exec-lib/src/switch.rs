use anyhow::Context;
use hyprland::data::Workspace;
use hyprland::default_instance_panic;
use hyprland::dispatch::{
    Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::{Address, WorkspaceId};
use tracing::{trace, warn};

pub fn switch_client(address: Address) -> anyhow::Result<()> {
    trace!("execute switch to client: {}", address);
    Dispatch::call(
        default_instance_panic(),
        DispatchType::FocusWindow(WindowIdentifier::Address(address)),
    )?;
    Dispatch::call(default_instance_panic(), DispatchType::BringActiveToTop)?;
    Ok(())
}

pub fn switch_client_by_initial_class(class: &str) -> anyhow::Result<()> {
    trace!("execute switch to client: {} by initial_class", class);
    Dispatch::call(
        default_instance_panic(),
        DispatchType::FocusWindow(WindowIdentifier::ClassRegularExpression(&format!(
            "initialclass:{}",
            class.to_ascii_lowercase()
        ))),
    )?;
    Dispatch::call(default_instance_panic(), DispatchType::BringActiveToTop)?;
    Ok(())
}

pub fn switch_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    // check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
    let current_workspace = Workspace::get_active(default_instance_panic());
    if let Ok(workspace) = current_workspace {
        if workspace_id == workspace.id {
            trace!("Already on workspace {}", workspace_id);
            return Ok(());
        }
    }

    if workspace_id < 0 {
        warn!(
            "Special workspace id detected, not switching to special workspace, TODO not supported"
        );
    } else {
        switch_normal_workspace(workspace_id).with_context(|| {
            format!("Failed to execute switch workspace with id {workspace_id}",)
        })?;
    }
    Ok(())
}

fn switch_normal_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    trace!("execute switch to workspace {workspace_id}");
    Dispatch::call(
        default_instance_panic(),
        DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(workspace_id)),
    )?;
    Ok(())
}
