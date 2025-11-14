use core_lib::transfer::Direction;
use core_lib::{
    Active, ClientData, ClientId, FindByFirst, GetFirstOrLast, HyprlandData, RevIf, WorkspaceData,
    WorkspaceId,
};
use std::cmp::{max, min};
use tracing::{debug, instrument, trace, trace_span, warn};

pub fn find_next_workspace(
    direction: &Direction,
    wrap: bool,
    hypr_data: &HyprlandData,
    active: Active,
    workspaces_per_row: u8,
) -> Active {
    let _span = trace_span!("find_next_workspace", direction = ?direction, wrap = wrap, active = ?active, workspaces_per_row = workspaces_per_row).entered();

    if hypr_data.workspaces.is_empty() {
        debug!("No workspaces available, returning None");
        return active;
    }
    let filtered = hypr_data
        .workspaces
        .iter()
        .filter(|(_, data)| data.any_client_enabled)
        .collect::<Vec<_>>();
    if filtered.len() == 1 {
        trace!("Only one workspaces available, returning current workspaces");
        let workspaces = &hypr_data.workspaces[0];
        return Active {
            client: None,
            workspace: workspaces.0,
            monitor: workspaces.1.monitor,
        };
    }
    let current = filtered
        .iter()
        .position(|(id, _)| *id == active.workspace)
        .unwrap_or_else(|| {
            warn!("Active workspace not found in workspaces, returning first workspace");
            0
        });

    let index = find_next_grid(direction, wrap, filtered.len(), current, workspaces_per_row);
    #[allow(clippy::map_unwrap_or)]
    let next_active = filtered
        .get(index)
        .map(|(id, data)| Active {
            client: None,
            workspace: *id,
            monitor: data.monitor,
        })
        .unwrap_or_else(|| {
            warn!("Unable to find next workspace, returning current workspace");
            active
        });
    // .expect("unable to find next workspace!");
    trace!("Next active: {next_active:?}");
    next_active
}

#[instrument(
    level = "trace",
    skip_all,
    ret,
    fields(direction = ?direction, wrap = wrap, active = ?active, clients_per_row = clients_per_row)
)]
pub fn find_next_client(
    direction: &Direction,
    wrap: bool,
    hypr_data: &HyprlandData,
    active: Active,
    clients_per_row: u8,
) -> Active {
    if hypr_data.clients.is_empty() {
        debug!("No clients available, returning None");
        return active;
    }

    #[allow(clippy::option_if_let_else)]
    let next_active = match active.client {
        None => find_first_client(direction, &hypr_data.clients, &hypr_data.workspaces, active),
        Some(client_id) => {
            let filtered = hypr_data
                .clients
                .iter()
                .filter(|(_, data)| data.enabled)
                .collect::<Vec<_>>();
            if filtered.len() == 1 {
                trace!("Only one client available, returning current client");
                let client = &hypr_data.clients[0];
                return Active {
                    client: Some(client.0),
                    workspace: client.1.workspace,
                    monitor: client.1.monitor,
                };
            }
            let current = filtered
                .iter()
                .position(|(id, _)| *id == client_id)
                .unwrap_or(0);
            let index = find_next_grid(direction, wrap, filtered.len(), current, clients_per_row);
            #[allow(clippy::map_unwrap_or)]
            filtered
                .get(index)
                .map(|(id, data)| Active {
                    client: Some(*id),
                    workspace: data.workspace,
                    monitor: data.monitor,
                })
                .unwrap_or_else(|| {
                    warn!("Unable to find next client, returning current client");
                    active
                })
        }
    };

    trace!("Next active: {next_active:?}");
    next_active
}

#[allow(clippy::cast_possible_wrap)] // wrapping wont happen here, number of workspaces or clients are way lower than isize::MAX
// #[instrument(level = "trace", skip_all, ret, fields(len = filtered_len, current = current))]
fn find_next_grid(
    direction: &Direction,
    wrap: bool,
    filtered_len: usize,
    current: usize,
    items_per_row: u8,
) -> usize {
    let _span = trace_span!("find_next_grid", len = filtered_len, current = current).entered();

    let i_per_row = isize::from(items_per_row);
    let items_per_row = usize::from(items_per_row);
    let offset = match direction {
        Direction::Right => 1,
        Direction::Left => -1,
        Direction::Up => -i_per_row,
        Direction::Down => i_per_row,
    };
    trace!("Finding next workspace with offset: {}", offset);
    let index = if wrap {
        let mut index = current as isize + offset;
        if index >= filtered_len as isize {
            trace!("Index out of bounds, wrapping around {index}");
            // subtract all rows / move to the beginning
            index -= (items_per_row * usize::div_ceil(filtered_len - 1, items_per_row)) as isize;
            if index < 0 {
                match direction {
                    Direction::Right => index = 0,
                    Direction::Down => {
                        index += i_per_row;
                    }
                    _ => unreachable!(),
                }
            }
        } else if index < 0 {
            trace!("Index out of bounds, wrapping around {index}");
            // add all rows / move to end
            index += (items_per_row * usize::div_ceil(filtered_len - 1, items_per_row)) as isize;
            if index >= filtered_len as isize {
                match direction {
                    Direction::Left => {
                        index = filtered_len as isize - 1;
                    }
                    Direction::Up => {
                        index -= i_per_row;
                    }
                    _ => unreachable!(),
                }
            }
        }
        #[allow(clippy::cast_sign_loss)] // index always positive, see if else above
        {
            index as usize
        }
    } else {
        #[allow(clippy::cast_sign_loss)]
        // max(current as isize + offset, 0) always positive (max function)
        {
            min(max(current as isize + offset, 0) as usize, filtered_len - 1)
        }
    };
    trace!("Next index: {index}");
    index
}

fn find_first_client(
    direction: &Direction,
    clients: &[(ClientId, ClientData)],
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: Active,
) -> Active {
    let get_last = matches!(direction, Direction::Left | Direction::Up);
    let (id, next) = clients
        .iter()
        .filter(|(_, c)| c.workspace == active.workspace && c.enabled)
        .get_first_or_last(get_last)
        .unwrap_or_else(|| {
            trace!("No client found in current workspace, looking for next client in workspaces");
            workspaces
                .iter()
                .reverse_if(get_last)
                .skip_while(|(id, _)| id != &active.workspace)
                .find_map(|(id, _)| {
                    trace!("Finding first client in workspace {id:?}");
                    clients
                        .iter()
                        .filter(|(_, c)| c.workspace == *id && c.enabled)
                        .get_first_or_last(get_last)
                })
                .unwrap_or_else(|| {
                    clients
                        .iter()
                        .filter(|(_, c)| c.monitor == active.monitor)
                        .get_first_or_last(get_last)
                        .unwrap_or_else(|| {
                            clients
                                .iter()
                                .get_first_or_last(get_last)
                                .expect("clients contain at least two clients")
                        })
                })
        });
    Active {
        client: Some(*id),
        workspace: next.workspace,
        monitor: next.monitor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn create_test_data(
        client_count: usize,
        workspace_count: usize,
        enabled: Option<usize>,
    ) -> (
        Vec<(ClientId, ClientData)>,
        Vec<(WorkspaceId, WorkspaceData)>,
    ) {
        let clients: Vec<(ClientId, ClientData)> = (0..client_count)
            .map(|i| {
                (
                    i as ClientId,
                    ClientData {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                        class: String::new(),
                        title: String::new(),
                        #[allow(clippy::cast_possible_wrap)]
                        workspace: (i % workspace_count) as WorkspaceId,
                        monitor: 0,
                        focus_history_id: 0,
                        floating: false,
                        #[allow(clippy::map_unwrap_or)]
                        enabled: enabled.map(|s| s >= i).unwrap_or(true),
                        pid: 0,
                    },
                )
            })
            .collect();

        #[allow(clippy::cast_possible_wrap)]
        let workspaces: Vec<(WorkspaceId, WorkspaceData)> = (0..workspace_count)
            .map(|i| {
                (
                    i as WorkspaceId,
                    WorkspaceData {
                        name: String::new(),
                        width: 0,
                        height: 0,
                        monitor: 0,
                        any_client_enabled: clients
                            .iter()
                            .any(|(_, c)| c.workspace == i as WorkspaceId && c.enabled),
                    },
                )
            })
            .collect();

        (clients, workspaces)
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_workspace_0() {
        let (clients, workspaces) = create_test_data(0, 0, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        assert_eq!(
            find_next_workspace(&Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Left, true, &hypr_data, active, 6),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Up, false, &hypr_data, active, 200),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Down, false, &hypr_data, active, 0),
            active
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_workspace_1_filter() {
        let (clients, workspaces) = create_test_data(2, 1, Some(0));
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        assert_eq!(
            find_next_workspace(&Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Left, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Up, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Down, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Right, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Left, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Up, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Down, false, &hypr_data, active, 3),
            active
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_workspace_1() {
        let (clients, workspaces) = create_test_data(1, 1, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        assert_eq!(
            find_next_workspace(&Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Left, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Up, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Down, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Right, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Left, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Up, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(&Direction::Down, false, &hypr_data, active, 3),
            active
        );
    }
    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_workspace_2() {
        let (clients, workspaces) = create_test_data(5, 5, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        let next = find_next_workspace(&Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(&Direction::Left, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 4);

        let next = find_next_workspace(&Direction::Up, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 3);
        let next = find_next_workspace(&Direction::Down, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 3);

        let next = find_next_workspace(&Direction::Right, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(&Direction::Left, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(&Direction::Up, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(&Direction::Down, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 3);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_workspace_2_filter() {
        let (clients, workspaces) = create_test_data(5, 5, Some(2));
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        let next = find_next_workspace(&Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(&Direction::Left, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 2);

        let next = find_next_workspace(&Direction::Up, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(&Direction::Down, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(&Direction::Right, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(&Direction::Left, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(&Direction::Up, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(&Direction::Down, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 2);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_client() {
        // Test with no clients
        let (clients, workspaces) = create_test_data(0, 1, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        trace!("data: {hypr_data:?}");

        assert_eq!(
            find_next_client(&Direction::Right, true, &hypr_data, active, 3),
            active
        );

        // Test with one client
        let (clients, workspaces) = create_test_data(1, 1, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        let next = find_next_client(&Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(
            next,
            Active {
                client: Some(0),
                workspace: 0,
                monitor: 0,
            }
        );

        // Test with multiple clients
        let (clients, workspaces) = create_test_data(4, 2, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        // Test with no active client
        let next = find_next_client(&Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.client, Some(0));

        // Test with active client
        let active = Active {
            client: Some(1),
            workspace: 1,
            monitor: 0,
        };

        // Test right direction with wrap
        let next = find_next_client(&Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.client, Some(2));

        // Test left direction with wrap
        let next = find_next_client(&Direction::Left, true, &hypr_data, active, 3);
        assert_eq!(next.client, Some(0));

        // Test without wrap
        let next = find_next_client(&Direction::Right, false, &hypr_data, active, 3);
        assert_eq!(next.client, Some(2));
    }
}

pub fn get_curr_client_pos(clients: &[(ClientId, ClientData)], active: ClientId) -> Option<usize> {
    let mut enabled_iter = clients.iter().filter(|(_, data)| data.enabled);
    if enabled_iter.clone().next().is_none() {
        debug!("Cannot get current client position. No enabled clients, returning None");
        return None;
    }

    enabled_iter.position(|(id, _)| *id == active)
}

pub fn get_curr_workspace_pos(
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: WorkspaceId,
) -> Option<usize> {
    if !workspaces.iter().any(|(_, data)| data.any_client_enabled) {
        debug!("Cannot get current workspace position. No enabled workspaces, returning None");
        return None;
    }

    workspaces.iter().position(|(id, _)| *id == active)
}

/// Determines which workspace should be active after a refresh.
///
/// After killing a workspace (all its clients), this selects the workspace at the same
/// position. If the workspace still has clients, it remains selected.
pub fn select_next_active_workspace(
    current_workspace_id: WorkspaceId,
    current_pos: Option<usize>,
    clients_data: &HyprlandData,
    active_prev: Active,
    items_per_row: usize,
) -> Active {
    // Check if current workspace still has enabled clients
    let workspace_still_valid = clients_data
        .workspaces
        .find_by_first(&current_workspace_id)
        .is_some_and(|w| w.any_client_enabled);

    if workspace_still_valid {
        // Workspace still has clients, keep it selected (non-kill refresh case)
        return Active {
            client: None,
            workspace: current_workspace_id,
            monitor: clients_data
                .workspaces
                .find_by_first(&current_workspace_id)
                .map_or(active_prev.monitor, |w| w.monitor),
        };
    }

    // Workspace was killed or lost all clients - select item at same position
    select_next_workspace_at_position(current_pos, clients_data, active_prev, items_per_row)
}

/// Selects the workspace at the given position.
/// Falls back to using `find_next` if position is unavailable or no workspaces remain.
fn select_next_workspace_at_position(
    position: Option<usize>,
    clients_data: &HyprlandData,
    active_prev: Active,
    items_per_row: usize,
) -> Active {
    let enabled_workspaces: Vec<_> = clients_data
        .workspaces
        .iter()
        .filter(|(_, w)| w.any_client_enabled)
        .collect();

    if enabled_workspaces.is_empty() {
        // No workspaces with clients remain
        return active_prev;
    }

    let Some(pos) = position else {
        // Couldn't determine position, use find_next to pick one
        return find_next_workspace(
            &Direction::Right,
            true,
            clients_data,
            active_prev,
            items_per_row as u8,
        );
    };

    // Select workspace at same position (clamped to list bounds)
    let new_pos = pos.min(enabled_workspaces.len() - 1);
    let (id, workspace) = enabled_workspaces[new_pos];
    Active {
        client: None,
        workspace: *id,
        monitor: workspace.monitor,
    }
}

/// Determines which client should be active after a refresh.
///
/// After killing a client, this selects the client at the same position.
/// If the client still exists, it remains selected.
pub fn select_next_active_client(
    current_client_id: Option<ClientId>,
    current_pos: Option<usize>,
    clients_data: &HyprlandData,
    active_prev: Active,
    items_per_row: usize,
) -> Active {
    // Check if we have a current client and if it still exists
    if let Some(client_id) = current_client_id {
        if clients_data.clients.find_by_first(&client_id).is_some() {
            // Current client still exists, keep it selected (non-kill refresh case)
            let client = clients_data.clients.find_by_first(&client_id).unwrap();
            return Active {
                client: Some(client_id),
                workspace: client.workspace,
                monitor: client.monitor,
            };
        }
    }

    // Client was killed or doesn't exist - select item at same position
    select_next_client_at_position(current_pos, clients_data, active_prev, items_per_row)
}

/// Selects the client at the given position.
/// Falls back to using `find_next` if position is unavailable or no clients remain.
fn select_next_client_at_position(
    position: Option<usize>,
    clients_data: &HyprlandData,
    active_prev: Active,
    items_per_row: usize,
) -> Active {
    let enabled_clients: Vec<_> = clients_data
        .clients
        .iter()
        .filter(|(_, c)| c.enabled)
        .collect();

    if enabled_clients.is_empty() {
        // No clients remain
        return active_prev;
    }

    let Some(pos) = position else {
        // Couldn't determine position, use find_next to pick one
        return find_next_client(
            &Direction::Right,
            true,
            clients_data,
            active_prev,
            items_per_row as u8,
        );
    };

    // Select client at same position (clamped to list bounds)
    let new_pos = pos.min(enabled_clients.len() - 1);
    let (id, client) = enabled_clients[new_pos];
    Active {
        client: Some(*id),
        workspace: client.workspace,
        monitor: client.monitor,
    }
}

// TODO add some tests here
