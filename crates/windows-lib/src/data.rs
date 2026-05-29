use crate::sort::{
    sort_clients_by_position, sort_clients_by_recent, sort_monitor_by_x,
    sort_workspaces_by_position, sort_workspaces_by_recent,
};
use core_lib::{
    Active, ByFirst, ClientData, ClientId, HyprlandData, MonitorData, MonitorId, WarnWithDetails,
};
use exec_lib::collect::collect_hypr_data;
use regex::Regex;
use tracing::{debug, debug_span, trace, warn};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct SortConfig {
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub exclude_workspaces: Option<Box<str>>,
}

pub fn collect_data(config: &SortConfig) -> anyhow::Result<(HyprlandData, Active)> {
    let _span = debug_span!("collect_data").entered();

    let exclude_workspaces = config.exclude_workspaces.as_ref().and_then(|reg| {
        let reg = Regex::new(&reg).warn_details("invalid regex");
        debug!("filtering special workspaces with regex: {reg:?}");
        reg
    });

    let (
        mut client_data,
        mut workspace_data,
        mut monitor_data,
        active_client,
        active_ws,
        active_monitor,
    ) = collect_hypr_data(exclude_workspaces)?;
    client_data = update_client_position(client_data, &monitor_data);
    sort_monitor_by_x(&mut monitor_data);
    if config.sort_recent {
        sort_clients_by_recent(&mut client_data);
        sort_workspaces_by_recent(&mut workspace_data, &client_data); // ! must be after sort_clients_by_recent
    } else {
        sort_workspaces_by_position(&mut workspace_data, &monitor_data); // ! must be before sort_clients_by_position
        client_data = sort_clients_by_position(client_data, &workspace_data, &monitor_data);
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
    for (id, ws) in &mut workspace_data {
        ws.any_client_enabled = {
            let mut filtered = client_data.iter().filter(|(_, c)| c.workspace.eq(id));
            *id == active_ws
                || filtered.clone().any(|(_, c)| c.enabled) && filtered.all(|(_, c)| c.enabled)
        };
    }

    trace!("client_data: {client_data:?}");
    trace!("workspace_data: {workspace_data:?}");
    trace!("monitor_data: {monitor_data:?}");

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

/// removes offset by monitor from clients
pub fn update_client_position(
    clients: Vec<(ClientId, ClientData)>,
    monitor_data: &[(MonitorId, MonitorData)],
) -> Vec<(ClientId, ClientData)> {
    clients
        .into_iter()
        .filter_map(|(a, mut c)| {
            let md = monitor_data.find_by_first(&c.monitor).or_else(|| {
                warn!("Monitor {:?} not found: {c:?}", c.monitor);
                None
            });

            if let Some(md) = md {
                c.x -= md.x as i16;
                c.y -= md.y as i16;
                Some((a, c))
            } else {
                None
            }
        })
        .collect()
}
