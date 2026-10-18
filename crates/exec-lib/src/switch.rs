use crate::util::to_client_address;
use crate::workspace_identity::{WorkspaceTarget, basic_key, full_key, resolve};
use anyhow::Context;
use core_lib::{ClientId, Warn, WorkspaceId};
use hyprland::data::{Client, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{
    Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
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
    // Validate the normal selector before any compositor IPC or fallback.
    // Negative legacy/special keys retain the existing special-workspace path.
    let target = resolve(workspace_id)?;
    if workspace_id >= 0 {
        normal_workspace_dispatch(target.clone())?;
    }
    if matches!(
        target,
        WorkspaceTarget::Named(_) | WorkspaceTarget::Special(_)
    ) {
        // An expired UI selection must not recreate a workspace or hide an open special workspace.
        workspace_by_key(workspace_id)?;
    }
    if workspace_id >= 0 {
        deactivate_special_workspace_if_needed().warn();
    }

    // check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
    let current_workspace = Workspace::get_active();
    if let Ok(workspace) = current_workspace
        && workspace_id == full_key(&workspace)?
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
    dispatch_with_legacy_fallback(
        "special workspace",
        || switch_special_workspace_lua(workspace_id),
        || switch_special_workspace_legacy(workspace_id),
    )
}

fn switch_special_workspace_legacy(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    let ws = workspace_by_key(workspace_id)?;
    if special_workspace_active(&ws)? {
        return Ok(());
    }

    Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(special_name(
        workspace_id,
        &ws,
    )?)))
    .context("failed to execute dispatch")?;
    Ok(())
}

fn switch_special_workspace_lua(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    let ws = workspace_by_key(workspace_id)?;
    if special_workspace_active(&ws)? {
        return Ok(());
    }

    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Special(Some(special_name(
            workspace_id,
            &ws,
        )?)),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

#[derive(Debug)]
struct ExpiredWorkspaceSelection;

impl std::fmt::Display for ExpiredWorkspaceSelection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("selected workspace no longer exists")
    }
}

impl std::error::Error for ExpiredWorkspaceSelection {}

fn dispatch_with_legacy_fallback(
    label: &str,
    primary: impl FnOnce() -> anyhow::Result<()>,
    legacy: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    match primary() {
        Err(error) if error.is::<ExpiredWorkspaceSelection>() => Err(error),
        Err(error) => {
            warn!("Failed to switch {label}: {}, trying legacy syntax", error);
            legacy()
        }
        Ok(result) => Ok(result),
    }
}

fn workspace_by_key(key: WorkspaceId) -> anyhow::Result<Workspace> {
    for workspace in Workspaces::get()? {
        if full_key(&workspace)? == key {
            return Ok(workspace);
        }
    }
    Err(ExpiredWorkspaceSelection.into())
}

fn special_name(key: WorkspaceId, workspace: &Workspace) -> anyhow::Result<String> {
    match resolve(key)? {
        WorkspaceTarget::Special(name) => Ok(name),
        WorkspaceTarget::Number(_) => Ok(workspace
            .name
            .strip_prefix("special:")
            .unwrap_or(&workspace.name)
            .to_owned()),
        WorkspaceTarget::Named(_) => anyhow::bail!("normal workspace passed to special dispatch"),
    }
}

fn special_workspace_active(workspace: &Workspace) -> anyhow::Result<bool> {
    let workspace_key = full_key(workspace)?;
    for monitor in Monitors::get()? {
        let special = monitor.special_workspace;
        if special.id.is_none() && special.address.is_empty() && special.name.is_empty() {
            continue;
        }
        if basic_key(&special)? == workspace_key {
            return Ok(true);
        }
    }
    Ok(false)
}

#[instrument(level = "debug", ret(level = "trace"))]
fn switch_normal_workspace(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    dispatch_with_legacy_fallback(
        "workspace",
        || switch_normal_workspace_lua(workspace_id),
        || switch_normal_workspace_legacy(workspace_id),
    )
}

fn switch_normal_workspace_lua(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    let target = resolve(workspace_id)?;
    if matches!(&target, WorkspaceTarget::Named(_)) {
        workspace_by_key(workspace_id)?;
    }
    let disp = normal_workspace_dispatch(target)?;
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

fn normal_workspace_dispatch(
    target: WorkspaceTarget,
) -> anyhow::Result<hyprland::dispatch_new::Dispatch> {
    use hyprland::dispatch_new::{Dispatch, WorkspaceIdentifier};
    let workspace = match target {
        WorkspaceTarget::Number(number) => {
            // Preserve the current unsigned range without wrapping through the
            // legacy signed ID API or reinterpreting a number as a name.
            if !(1..=i64::from(u32::MAX)).contains(&number) {
                anyhow::bail!(
                    "numeric workspace cannot be dispatched by the current compositor selector grammar: {number}"
                );
            }
            WorkspaceIdentifier::Number(u32::try_from(number)?)
        }
        WorkspaceTarget::Named(address) => {
            // Selected Hyprland's resolver rejects an empty name and ASCII
            // integer names, including a leading minus. Keep their collection
            // identities distinct, but never dispatch an unsupported selector.
            let digits = address.strip_prefix('-').unwrap_or(&address);
            if address.is_empty()
                || (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
            {
                anyhow::bail!(
                    "named workspace cannot be dispatched by the current compositor selector grammar: {address}"
                );
            }
            // The compositor's name: selector resolves the immutable address,
            // while workspace.name is only the mutable display label.
            WorkspaceIdentifier::Name(address)
        }
        WorkspaceTarget::Special(_) => anyhow::bail!("special workspace passed to normal dispatch"),
    };
    Ok(Dispatch::FocusWorkspace(workspace, false))
}

fn switch_normal_workspace_legacy(workspace_id: WorkspaceId) -> anyhow::Result<()> {
    debug!("execute switch to workspace {workspace_id}");
    let target = resolve(workspace_id)?;
    if let WorkspaceTarget::Number(number) = &target {
        return Dispatch::call(DispatchType::Custom("workspace", &number.to_string()))
            .context("failed to execute numeric workspace dispatch");
    }
    let dispatch = match &target {
        WorkspaceTarget::Number(_) => unreachable!(),
        WorkspaceTarget::Named(address) => {
            workspace_by_key(workspace_id)?;
            WorkspaceIdentifierWithSpecial::Name(address)
        }
        WorkspaceTarget::Special(_) => anyhow::bail!("special workspace passed to normal dispatch"),
    };
    Dispatch::call(DispatchType::Workspace(dispatch)).context("failed to execute dispatch")?;
    Ok(())
}

/// always run when changing client or workspace
///
/// if client on special workspace is opened the workspace is activated
#[instrument(level = "debug", ret(level = "trace"))]
fn deactivate_special_workspace_if_needed() -> anyhow::Result<()> {
    let workspace = Workspace::get_active().context("active workspace failed")?;
    let (target, legacy_name) =
        if let Some(client) = Client::get_active().context("active client failed")? {
            (
                resolve(basic_key(&client.workspace)?)?,
                client.workspace.name,
            )
        } else {
            (resolve(full_key(&workspace)?)?, workspace.name)
        };
    let name = match target {
        WorkspaceTarget::Special(name) => Some(name),
        WorkspaceTarget::Number(number) if number < 0 => {
            legacy_name.strip_prefix("special:").map(str::to_owned)
        }
        _ => None,
    };
    if let Some(name) = name {
        dispatch_with_legacy_fallback(
            "deactivate special workspace",
            || deactivate_special_workspace_if_needed_lua(&name),
            || deactivate_special_workspace_if_needed_legacy(&name),
        )?;
    }
    Ok(())
}

fn deactivate_special_workspace_if_needed_lua(name: &str) -> anyhow::Result<()> {
    debug!("execute switch to workspace {name}");
    let disp = hyprland::dispatch_new::Dispatch::FocusWorkspace(
        hyprland::dispatch_new::WorkspaceIdentifier::Special(Some(name.to_owned())),
        false,
    );
    disp.apply().context("failed to execute dispatch")?;
    Ok(())
}

fn deactivate_special_workspace_if_needed_legacy(name: &str) -> anyhow::Result<()> {
    debug!("execute switch to workspace {name}");
    Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(name.to_owned())))?;
    Ok(())
}

#[cfg(test)]
mod workspace_dispatch_tests {
    use super::{
        ExpiredWorkspaceSelection, WorkspaceTarget, dispatch_with_legacy_fallback,
        normal_workspace_dispatch, switch_workspace,
    };
    use std::cell::Cell;

    #[test]
    fn expired_selection_never_invokes_legacy_dispatch() {
        for label in ["workspace", "special workspace"] {
            let primary_calls = Cell::new(0);
            let legacy_calls = Cell::new(0);
            let result = dispatch_with_legacy_fallback(
                label,
                || {
                    primary_calls.set(primary_calls.get() + 1);
                    Err(anyhow::Error::new(ExpiredWorkspaceSelection).context("actual dispatch"))
                },
                || {
                    legacy_calls.set(legacy_calls.get() + 1);
                    Ok(())
                },
            );
            assert!(
                result
                    .expect_err("invalid test fixture")
                    .is::<ExpiredWorkspaceSelection>()
            );
            assert_eq!(primary_calls.get(), 1);
            assert_eq!(legacy_calls.get(), 0);
        }
    }

    #[test]
    fn compatibility_failure_retains_legacy_dispatch_and_result() {
        for label in ["workspace", "special workspace"] {
            for legacy_succeeds in [true, false] {
                let primary_calls = Cell::new(0);
                let legacy_calls = Cell::new(0);
                let result = dispatch_with_legacy_fallback(
                    label,
                    || {
                        primary_calls.set(primary_calls.get() + 1);
                        anyhow::bail!("Lua API unavailable")
                    },
                    || {
                        legacy_calls.set(legacy_calls.get() + 1);
                        if legacy_succeeds {
                            Ok(())
                        } else {
                            anyhow::bail!("actual legacy failure")
                        }
                    },
                );
                assert_eq!(primary_calls.get(), 1);
                assert_eq!(legacy_calls.get(), 1);
                if legacy_succeeds {
                    assert!(result.is_ok());
                } else {
                    assert_eq!(
                        result.expect_err("invalid test fixture").to_string(),
                        "actual legacy failure"
                    );
                }
            }
        }
    }

    #[test]
    fn actual_focus_dispatch_preserves_supported_numbers_and_name_grammar() {
        for (target, expected) in [
            (
                WorkspaceTarget::Number(5),
                r#"hl.dsp.focus({ workspace = "5",  on_current_monitor = false, })"#,
            ),
            (
                WorkspaceTarget::Number(i64::from(i32::MAX)),
                r#"hl.dsp.focus({ workspace = "2147483647",  on_current_monitor = false, })"#,
            ),
            (
                WorkspaceTarget::Number(i64::from(u32::MAX)),
                r#"hl.dsp.focus({ workspace = "4294967295",  on_current_monitor = false, })"#,
            ),
            (
                WorkspaceTarget::Named("work".to_owned()),
                r#"hl.dsp.focus({ workspace = "name:work",  on_current_monitor = false, })"#,
            ),
        ] {
            assert_eq!(
                normal_workspace_dispatch(target)
                    .expect("valid test fixture")
                    .to_string(),
                expected
            );
        }
        for unsupported in [0, -1, i64::from(u32::MAX) + 1] {
            let error = normal_workspace_dispatch(WorkspaceTarget::Number(unsupported))
                .expect_err("invalid test fixture");
            assert!(
                error
                    .to_string()
                    .contains("current compositor selector grammar")
            );
            if unsupported == 0 {
                // Execute the real entrypoint: this must reject before its IPC
                // operations or the legacy fallback can observe the target.
                assert!(
                    switch_workspace(unsupported)
                        .expect_err("invalid test fixture")
                        .to_string()
                        .contains("current compositor selector grammar")
                );
            }
        }
        for address in ["5.2", "+5", "-", "-work"] {
            assert!(normal_workspace_dispatch(WorkspaceTarget::Named(address.to_owned())).is_ok());
        }
        for address in ["", "5", "-5", "005", "-0005"] {
            let error = normal_workspace_dispatch(WorkspaceTarget::Named(address.to_owned()))
                .expect_err("invalid test fixture");
            assert!(
                error
                    .to_string()
                    .contains("current compositor selector grammar")
            );
            if !address.is_empty() {
                let basic = hyprland::data::WorkspaceBasic {
                    id: None,
                    address: address.to_owned(),
                    workspace_type: "normal".to_owned(),
                    name: address.to_owned(),
                };
                let key = crate::workspace_identity::basic_key(&basic).expect("valid test fixture");
                assert!(key > i64::from(u32::MAX));
                // Execute the actual entrypoint without a running compositor:
                // unsupported names must fail before any IPC or fallback.
                assert!(
                    switch_workspace(key)
                        .expect_err("invalid test fixture")
                        .to_string()
                        .contains("current compositor selector grammar")
                );
            }
        }
        assert!(normal_workspace_dispatch(WorkspaceTarget::Special("5".to_owned())).is_err());
    }
}
