use crate::sort::{
    sort_clients_by_position, sort_clients_by_recent, sort_workspaces_by_position,
    sort_workspaces_by_recent,
};
use core_lib::{
    Active, ClientData, ClientId, FindByFirst, HyprlandData, MonitorData, MonitorId, WorkspaceData,
    WorkspaceId,
};
use exec_lib::collect::collect_hypr_data;
use tracing::{debug_span, trace, warn};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct SortConfig {
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
}

pub fn collect_data(config: &SortConfig) -> anyhow::Result<(HyprlandData, Active)> {
    let _span = debug_span!("collect_data").entered();

    let (
        mut client_data,
        mut workspace_data,
        monitor_data,
        active_client,
        active_ws,
        active_monitor,
    ) = collect_hypr_data()?;
    client_data = update_client_position(client_data, &workspace_data, &monitor_data);
    if config.sort_recent {
        sort_clients_by_recent(&mut client_data);
        sort_workspaces_by_recent(&mut workspace_data, &client_data); // ! must be after sort_clients_by_recent
    } else {
        client_data = sort_clients_by_position(client_data);
        sort_workspaces_by_position(&mut workspace_data);
    }

    trace!(
        "active_client: {active_client:?}; active_ws: {active_ws:?}; active_monitor: {active_monitor:?}"
    );

    // iterate over all clients and set active to false if the client is not on the active workspace or monitor
    for (_, client) in &mut client_data {
        client.enabled = (!config.filter_same_class
            || active_client
                .as_ref()
                .is_none_or(|active| client.class == *active.0))
            && (!config.filter_current_workspace || client.workspace == active_ws)
            && (!config.filter_current_monitor || client.monitor == active_monitor);
    }

    trace!("client_data: {:?}", client_data);
    trace!("workspace_data: {:?}", workspace_data);
    trace!("monitor_data: {:?}", monitor_data);

    Ok((
        HyprlandData {
            clients: client_data,
            workspaces: workspace_data,
            monitors: monitor_data,
        },
        Active {
            client: active_client.map(|c| c.1),
            workspace: active_ws,
            monitor: active_monitor,
        },
    ))
}

/// updates clients with workspace and monitor data
/// * clients - Vector of clients to update
/// * `workspace_data` - `HashMap` of workspace data
/// * `monitor_data` - `HashMap` of monitor data, None if `ignore_monitors`
///
/// removes offset by monitor, adds offset by workspace (client on monitor 1 and workspace 2 will be moved left by monitor 1 offset and right by workspace 2 offset (workspace width * 2))
pub fn update_client_position(
    clients: Vec<(ClientId, ClientData)>,
    workspace_data: &[(WorkspaceId, WorkspaceData)],
    monitor_data: &[(MonitorId, MonitorData)],
) -> Vec<(ClientId, ClientData)> {
    clients
        .into_iter()
        .filter_map(|(a, mut c)| {
            let ws = workspace_data
                .find_by_first(&c.workspace)
                .map(|ws| (ws.x, ws.y))
                .or_else(|| {
                    warn!("Workspace {:?} not found for client: {:?}", c.workspace, c);
                    None
                });

            let md = monitor_data
                .find_by_first(&c.monitor)
                .map(|md| (md.x, md.y))
                .or_else(|| {
                    warn!("Monitor {:?} not found: {:?}", c.monitor, c);
                    None
                });

            if let (Some((ws_x, ws_y)), Some((md_x, md_y))) = (ws, md) {
                c.x += (ws_x - md_x) as i16; // move x cord by workspace offset
                c.y += (ws_y - md_y) as i16; // move y cord by workspace offset
                Some((a, c))
            } else {
                None
            }
        })
        .collect()
}
