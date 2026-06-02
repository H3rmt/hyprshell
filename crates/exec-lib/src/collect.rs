use crate::util::to_client_id;
use anyhow::Context;
use core_lib::{ByFirst, ClientData, ClientId, MonitorData, MonitorId, WorkspaceData, WorkspaceId};
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace, Workspaces};
use hyprland::prelude::*;
use regex::Regex;
use tracing::{debug_span, instrument, trace, warn};

fn get_hypr_data() -> anyhow::Result<(Vec<Monitor>, Vec<Workspace>, Vec<Client>)> {
    let _span = debug_span!("get_hypr_data").entered();
    let monitors = Monitors::get().context("monitors failed")?.to_vec();
    // sort and filter all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get()
            .context("workspaces failed")?
            .into_iter()
            .filter(|w| w.id != -1) // TODO: check if still needed: ignore clients on invalid workspaces
            .collect::<Vec<_>>();

        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    };
    let clients = Clients::get()
        .context("clients failed")?
        .into_iter()
        .filter(|c| c.workspace.id != -1) // TODO: check if still needed: ignore clients on invalid workspaces
        .collect::<Vec<_>>();

    Ok((monitors, workspaces, clients))
}

#[allow(clippy::type_complexity, clippy::too_many_lines)]
#[instrument(level = "debug")]
pub fn collect_hypr_data(
    exclude_workspaces: Option<&Regex>,
) -> anyhow::Result<(
    Vec<(ClientId, ClientData)>,
    Vec<(WorkspaceId, WorkspaceData)>,
    Vec<(MonitorId, MonitorData)>,
    Option<(String, ClientId)>,
    WorkspaceId,
    MonitorId,
)> {
    let (monitors, workspaces, clients) =
        get_hypr_data().context("loading hyprland data failed")?;

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor.
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let mut monitor_data = {
        let mut md: Vec<(MonitorId, MonitorData)> = Vec::with_capacity(monitors.iter().len());

        for monitor in &monitors {
            #[allow(clippy::cast_sign_loss)]
            md.push((
                monitor.id,
                MonitorData {
                    id: monitor.id,
                    x: monitor.x,
                    y: monitor.y,
                    width: (f32::from(monitor.width) / monitor.scale) as u16,
                    height: (f32::from(monitor.height) / monitor.scale) as u16,
                    connector: monitor.name.clone(),
                    scale: monitor.scale,
                },
            ));
        }
        md
    };

    // all workspaces with their data, x and y are the offset of the workspace
    let mut workspace_data = {
        let mut wd: Vec<(WorkspaceId, WorkspaceData)> = Vec::with_capacity(workspaces.len());

        for (monitor_id, monitor_data) in &monitor_data {
            workspaces
                .iter()
                .filter(|ws| ws.monitor_id == Some(*monitor_id))
                .for_each(|workspace| {
                    wd.push((
                        workspace.id,
                        WorkspaceData {
                            name: workspace.name.clone(),
                            monitor: *monitor_id,
                            height: monitor_data.height,
                            width: monitor_data.width,
                            any_client_enabled: true, // gets updated later
                        },
                    ));
                });
        }
        wd
    };

    let mut client_data = {
        let mut cd: Vec<(ClientId, ClientData)> = Vec::with_capacity(clients.len());

        for client in clients {
            let Some(monitor) = client.monitor else {
                continue;
            };
            if workspace_data.find_by_first(&client.workspace.id).is_some() {
                cd.push((
                    to_client_id(&client.address),
                    ClientData {
                        x: client.at.0,
                        y: client.at.1,
                        width: client.size.0,
                        height: client.size.1,
                        class: client.class.clone(),
                        workspace: client.workspace.id,
                        monitor,
                        focus_history_id: client.focus_history_id,
                        title: client.title.clone(),
                        floating: client.floating,
                        pid: client.pid,
                        enabled: true, // gets updated later
                    },
                ));
            } else {
                warn!(
                    "workspace {:?} not found for client {client:?}",
                    client.workspace
                );
            }
        }
        cd
    };

    // we do this after the initial collection because then clients would complain
    // about missing workspace
    trace!(
        "workspaces bevore filter by regex: {}",
        workspace_data.len()
    );
    workspace_data.retain(|(_, ws)| {
        exclude_workspaces
            .as_ref()
            .is_none_or(|reg| !reg.is_match(&ws.name))
    });
    trace!("workspaces after filter by regex: {}", workspace_data.len());
    client_data.retain(|(_id, cl)| workspace_data.find_by_first(&cl.workspace).is_some());

    workspace_data.sort_by(|a, b| a.0.cmp(&b.0));
    monitor_data.sort_by(|a, b| a.0.cmp(&b.0));

    // is broken, reports the "normal" workspace as active when a client in special workspace is selected
    // let active_ws = Workspace::get_active()?.id;
    let active_ws = Workspace::get_active()
        .map(|w| w.id)
        .context("active workspace failed")?;
    let active_ws = Client::get_active()
        .context("active client failed")?
        .map_or(active_ws, |a| a.workspace.id);
    let active_monitor = Monitor::get_active().context("active monitor failed")?.id;
    let active_client = Client::get_active()
        .context("active client failed")?
        .map(|a| (a.class.clone(), to_client_id(&a.address)));

    Ok((
        client_data,
        workspace_data,
        monitor_data,
        active_client,
        active_ws,
        active_monitor,
    ))
}

pub fn get_monitors() -> Vec<MonitorData> {
    Monitors::get()
        .map_or(vec![], HyprDataVec::to_vec)
        .iter()
        .map(|m| MonitorData {
            id: m.id,
            x: m.x,
            y: m.y,
            width: m.width,
            height: m.height,
            connector: m.name.clone(),
            scale: m.scale,
        })
        .collect()
}

#[must_use]
pub fn get_current_monitor() -> Option<MonitorData> {
    Monitor::get_active().ok().map(|m| MonitorData {
        id: m.id,
        x: m.x,
        y: m.y,
        width: m.width,
        height: m.height,
        connector: m.name.clone(),
        scale: m.scale,
    })
}

pub fn get_client_classes() -> Vec<String> {
    Clients::get()
        .map_or(vec![], HyprDataVec::to_vec)
        .iter()
        .map(|client| client.class.clone())
        .collect()
}
