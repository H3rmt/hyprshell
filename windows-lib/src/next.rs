use anyhow::Context;
use core_lib::transfer::Direction;
use core_lib::{Active, ClientData, ClientId, HyprlandData, WorkspaceData, WorkspaceId};
use tracing::{Level, debug, span, trace};

pub fn find_next(
    direction: &Direction,
    workspace: bool,
    hypr_data: &HyprlandData,
    active: Active,
    workspaces_per_row: usize,
) -> Active {
    let _span =
        span!(Level::TRACE, "find_next", direction = ?direction, workspace, active = ?active)
            .entered();
    match (workspace, direction) {
        (false, dir) => {
            // get first client on workspace or monitor
            let dat: Option<(ClientId, ClientData)> = if let Some(id) = active.client {
                find_next_client(dir, &hypr_data.clients, id, workspaces_per_row).ok()
            } else {
                trace!("No active client, getting first client");
                let mut clients = hypr_data
                    .clients
                    .iter()
                    .filter(|(_, c)| c.workspace == active.workspace)
                    .map(|(id, c)| (*id, c.clone()))
                    .collect::<Vec<_>>();
                if clients.is_empty() {
                    clients = hypr_data
                        .clients
                        .iter()
                        .filter(|(_, c)| c.monitor == active.monitor)
                        .map(|(id, c)| (*id, c.clone()))
                        .collect::<Vec<_>>();
                }
                if clients.is_empty() {
                    clients = hypr_data
                        .clients
                        .iter()
                        .map(|(id, c)| (*id, c.clone()))
                        .collect::<Vec<_>>();
                }
                match dir {
                    Direction::Right | Direction::Down => Some(clients.first().cloned()),
                    Direction::Left | Direction::Up => Some(clients.last().cloned()),
                }
                .unwrap_or(None)
            };

            debug!("Next client: {:?}", dat.as_ref().map(|(id, _)| *id));
            Active {
                client: dat.as_ref().map(|(id, _)| *id),
                workspace: dat
                    .as_ref()
                    .map(|(_, dat)| dat.workspace)
                    .unwrap_or(active.workspace),
                monitor: dat.map(|(_, dat)| dat.monitor).unwrap_or(active.monitor),
            }
        }
        (true, dir) => find_next_workspace(
            dir,
            &hypr_data.workspaces,
            active.workspace,
            workspaces_per_row,
        )
        .map(|(workspace_id, workspace_data)| {
            debug!("Next workspace: {:?}", workspace_id);
            Active {
                client: None,
                workspace: workspace_id,
                monitor: workspace_data.monitor,
            }
        })
        .unwrap_or(active),
    }
}

pub(crate) fn find_next_workspace(
    direction: &Direction,
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: WorkspaceId,
    workspaces_per_row: usize,
) -> anyhow::Result<(WorkspaceId, WorkspaceData)> {
    let workspaces = workspaces
        .iter()
        .filter(|(_, c)| c.enabled)
        .map(|(id, workspace)| (*id, workspace.clone()))
        .collect::<Vec<_>>();

    let ind = workspaces.iter().position(|(id, _)| *id == active);
    trace!("Current index: {:?} in len: {}", ind, workspaces.len());
    let index: i64 = match ind {
        Some(si) => match direction {
            Direction::Right => si as i64 + 1,
            Direction::Left => {
                if si == 0 {
                    workspaces.len() as i64 - 1
                } else {
                    si as i64 - 1
                }
            }
            Direction::Up => si as i64 - workspaces_per_row as i64,
            Direction::Down => {
                if si as i64 >= workspaces.len() as i64 - workspaces_per_row as i64 {
                    workspaces.len() as i64 - 1
                } else {
                    si as i64 + workspaces_per_row as i64
                }
            }
        },
        None => match direction {
            Direction::Left | Direction::Up => workspaces.len() as i64 - 1,
            Direction::Right | Direction::Down => 0,
        },
    };
    trace!("New index: {}", index);
    let len = workspaces.len() as i64;

    let next_workspace = workspaces
        .into_iter()
        .cycle()
        .nth(index.rem_euclid(len) as usize)
        .context("No next client found")?;

    Ok(next_workspace)
}

pub(crate) fn find_next_client(
    direction: &Direction,
    clients: &[(ClientId, ClientData)],
    active: ClientId,
    clients_per_row: usize,
) -> anyhow::Result<(ClientId, ClientData)> {
    let clients = clients
        .iter()
        .filter(|(_, c)| c.enabled)
        .map(|(id, client)| (*id, client.clone()))
        .collect::<Vec<_>>();

    let ind = clients.iter().position(|(id, _)| *id == active);
    let index = match ind {
        Some(si) => match direction {
            Direction::Right => si + 1,
            Direction::Left => {
                if si == 0 {
                    clients.len() - 1
                } else {
                    si - 1
                }
            }
            Direction::Up => si.saturating_sub(clients_per_row),
            Direction::Down => {
                if si >= clients.len() - clients_per_row {
                    clients.len() - 1
                } else {
                    si + clients_per_row
                }
            }
        },
        None => match direction {
            Direction::Left | Direction::Up => clients.len() - 1,
            Direction::Right | Direction::Down => 0,
        },
    };

    let next_client = clients
        .into_iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok(next_client)
}
