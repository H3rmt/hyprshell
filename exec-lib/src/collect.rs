use crate::to_client_id;
use core_lib::{
    ClientData, ClientId, FindByFirst, MonitorData, MonitorId, WorkspaceData, WorkspaceId,
};
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace, Workspaces};
use hyprland::default_instance_panic;
use hyprland::prelude::*;
use tracing::{Level, span, warn};

fn get_hypr_data() -> anyhow::Result<(Vec<Monitor>, Vec<Workspace>, Vec<Client>)> {
    let instance = default_instance_panic();

    let _span = span!(Level::TRACE, "get_hypr_data").entered();
    let monitors = Monitors::get(instance)?.to_vec();
    // sort and filter all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get(instance)?
            .into_iter()
            .filter(|w| w.id != -1) // filter invalid workspaces
            .filter(|w| !w.id < 0) // TODO someday add special_workspace support
            .collect::<Vec<_>>();

        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    };
    let clients = Clients::get(instance)?
        .into_iter()
        .filter(|c| c.workspace.id != -1) // ignore clients on invalid workspaces
        .filter(|w| !w.workspace.id < 0) // TODO someday add special_workspace support
        .collect::<Vec<_>>();

    Ok((monitors, workspaces, clients))
}

#[allow(clippy::type_complexity)]
pub fn collect_hypr_data() -> anyhow::Result<(
    Vec<(ClientId, ClientData)>,
    Vec<(WorkspaceId, WorkspaceData)>,
    Vec<(MonitorId, MonitorData)>,
    Option<(String, ClientId)>,
    WorkspaceId,
    MonitorId,
)> {
    let _span = span!(Level::TRACE, "convert_hypr_data").entered();

    let (monitors, workspaces, clients) = get_hypr_data()?;

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor.
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let mut monitor_data = {
        let mut md: Vec<(MonitorId, MonitorData)> = Vec::with_capacity(monitors.iter().len());

        monitors.iter().for_each(|monitor| {
            md.push((
                monitor.id,
                MonitorData {
                    x: monitor.x,
                    y: monitor.y,
                    width: (monitor.width as f32 / monitor.scale) as u16,
                    height: (monitor.height as f32 / monitor.scale) as u16,
                    connector: monitor.name.clone(),
                },
            ));
        });
        md
    };

    // all workspaces with their data, x and y are the offset of the workspace
    let mut workspace_data = {
        let mut wd: Vec<(WorkspaceId, WorkspaceData)> = Vec::with_capacity(workspaces.len());

        for (monitor_id, monitor_data) in monitor_data.iter() {
            let mut x_offset: i32 = 0;
            workspaces
                .iter()
                .filter(|ws| ws.monitor_id == Some(*monitor_id))
                .for_each(|workspace| {
                    wd.push((
                        workspace.id,
                        WorkspaceData {
                            x: x_offset,
                            y: monitor_data.y,
                            name: workspace.name.clone(),
                            monitor: *monitor_id,
                            height: monitor_data.height,
                            width: monitor_data.width,
                        },
                    ));
                    x_offset += monitor_data.width as i32;
                });
        }
        wd
    };

    let client_data = {
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
                        enabled: false, // gets updated later
                    },
                ));
            } else {
                warn!(
                    "workspace {:?} not found for client {:?}",
                    client.workspace, client
                );
            }
        }
        cd
    };

    workspace_data.sort_by(|a, b| a.0.cmp(&b.0));
    monitor_data.sort_by(|a, b| a.0.cmp(&b.0));

    let instance = default_instance_panic();
    let active_ws = Workspace::get_active(instance)?.id;
    let active_monitor = Monitor::get_active(instance)?.id;
    let active_client =
        Client::get_active(instance)?.map(|a| (a.class.clone(), to_client_id(&a.address)));

    Ok((
        client_data,
        workspace_data,
        monitor_data,
        active_client,
        active_ws,
        active_monitor,
    ))
}
