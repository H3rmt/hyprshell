use core_lib::Direction;
use core_lib::{
    Active, ClientData, ClientId, GetFirstOrLast, HyprlandData, RevIf, WorkspaceData, WorkspaceId,
};
use std::cmp::{max, min};
use tracing::{debug, instrument, trace, trace_span, warn};

pub fn find_next_workspace(
    direction: Direction,
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
    direction: Direction,
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
    direction: Direction,
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
    let mut index = current as isize + offset;
    if index >= filtered_len as isize || index < 0 {
        if wrap {
            trace!("Index out of bounds, wrapping around {index}");
            match direction {
                Direction::Right | Direction::Left => {
                    index = wrap_sequence(index, filtered_len as isize - 1);
                }
                Direction::Up | Direction::Down => {
                    let row_count = usize::div_ceil(filtered_len, items_per_row);
                    let grid_size = (items_per_row * row_count) as isize;
                    index = wrap_grid(index, filtered_len as isize - 1, grid_size, i_per_row);
                }
            }
        } else {
            trace!("Index out of bounds, clamping {index} to avoid wrapping");
            match direction {
                Direction::Right | Direction::Left => {
                    index = min(max(index, 0), filtered_len as isize - 1);
                }
                Direction::Up | Direction::Down => {
                    index = current as isize;
                }
            }
        }
    }
    trace!("Next index: {index}");
    #[allow(clippy::cast_sign_loss)] // index always positive, see if else above
    {
        index as usize
    }
}

// When i goes out of bounds, simply wrap around to the other end of
// the sequence.
const fn wrap_sequence(i: isize, max: isize) -> isize {
    if i > max {
        0
    } else if i < 0 {
        max
    } else {
        i
    }
}

// When i goes out of bounds, wrap around to the other end. This time,
// however, keep i % row_len the same so that the selection stays in
// the same column of the grid on the screen.
#[instrument(level = "trace", ret)]
fn wrap_grid(i: isize, max: isize, grid_size: isize, row_len: isize) -> isize {
    let mut i = i;
    if i >= max {
        // subtract all rows / move to the beginning
        i -= grid_size;
        if i < 0 {
            i += row_len;
        }
    } else if i < 0 {
        // add all rows / move to end
        i += grid_size;
        if i > max {
            i -= row_len;
        }
    }
    i
}

fn find_first_client(
    direction: Direction,
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
            find_next_workspace(Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Left, true, &hypr_data, active, 6),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Up, false, &hypr_data, active, 200),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Down, false, &hypr_data, active, 0),
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
            find_next_workspace(Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Left, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Up, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Down, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Right, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Left, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Up, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Down, false, &hypr_data, active, 3),
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
            find_next_workspace(Direction::Right, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Left, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Up, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Down, true, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Right, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Left, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Up, false, &hypr_data, active, 3),
            active
        );
        assert_eq!(
            find_next_workspace(Direction::Down, false, &hypr_data, active, 3),
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

        let next = find_next_workspace(Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(Direction::Left, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 4);

        let next = find_next_workspace(Direction::Up, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 3);
        let next = find_next_workspace(Direction::Down, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 3);

        let next = find_next_workspace(Direction::Right, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(Direction::Left, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(Direction::Up, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(Direction::Down, false, &hypr_data, active, 3);
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

        let next = find_next_workspace(Direction::Right, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(Direction::Left, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 2);

        let next = find_next_workspace(Direction::Up, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(Direction::Down, true, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(Direction::Right, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 1);
        let next = find_next_workspace(Direction::Left, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);

        let next = find_next_workspace(Direction::Up, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
        let next = find_next_workspace(Direction::Down, false, &hypr_data, active, 3);
        assert_eq!(next.workspace, 0);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_client_empty() {
        // Test with no clients
        let (clients, workspaces) = create_test_data(0, 1, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        assert_eq!(
            find_next_client(Direction::Right, true, &hypr_data, active, 3),
            active
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_client_single() {
        // Test with one client
        let (clients, workspaces) = create_test_data(1, 1, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        let active = Active {
            client: None,
            workspace: 0,
            monitor: 0,
        };
        assert_eq!(
            find_next_client(Direction::Left, true, &hypr_data, active, 3),
            Active {
                client: Some(0),
                workspace: 0,
                monitor: 0,
            }
        );
        assert_eq!(
            find_next_client(Direction::Right, true, &hypr_data, active, 3),
            Active {
                client: Some(0),
                workspace: 0,
                monitor: 0,
            }
        );
        assert_eq!(
            find_next_client(Direction::Up, true, &hypr_data, active, 3),
            Active {
                client: Some(0),
                workspace: 0,
                monitor: 0,
            }
        );
        assert_eq!(
            find_next_client(Direction::Down, true, &hypr_data, active, 3),
            Active {
                client: Some(0),
                workspace: 0,
                monitor: 0,
            }
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_client_sequence() {
        fn current(idx: u64) -> Active {
            Active {
                client: Some(idx),
                workspace: 1,
                monitor: 0,
            }
        }

        // Test with multiple clients
        let (clients, workspaces) = create_test_data(4, 2, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        // Test right direction with wrapping
        let next = find_next_client(Direction::Right, true, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Right, true, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Right, true, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Right, true, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(0));

        // Test left direction with wrapping
        let next = find_next_client(Direction::Left, true, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Left, true, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Left, true, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Left, true, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(0));

        // Test right direction without wrapping
        let next = find_next_client(Direction::Right, false, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Right, false, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Right, false, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Right, false, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(3));

        // Test left direction without wrapping
        let next = find_next_client(Direction::Left, false, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(0));
        let next = find_next_client(Direction::Left, false, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Left, false, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Left, false, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(0));
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_find_next_client_grid() {
        fn current(idx: u64) -> Active {
            Active {
                client: Some(idx),
                workspace: 1,
                monitor: 0,
            }
        }

        // Test with multiple clients
        let (clients, workspaces) = create_test_data(8, 2, None);
        let hypr_data = HyprlandData {
            clients,
            workspaces,
            monitors: vec![],
        };
        trace!("data: {hypr_data:?}");

        // Test down direction with wrapping
        let next = find_next_client(Direction::Down, true, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(4));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(5));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(6));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(4), 3);
        assert_eq!(next.client, Some(7));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(5), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(6), 3);
        assert_eq!(next.client, Some(0));
        let next = find_next_client(Direction::Down, true, &hypr_data, current(7), 3);
        assert_eq!(next.client, Some(1));

        // Test up direction with wrapping
        let next = find_next_client(Direction::Up, true, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(6));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(7));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(5));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(0));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(4), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(5), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(6), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Up, true, &hypr_data, current(7), 3);
        assert_eq!(next.client, Some(4));

        // Test down direction without wrapping
        let next = find_next_client(Direction::Down, false, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(4));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(5));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(6));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(4), 3);
        assert_eq!(next.client, Some(7));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(5), 3);
        assert_eq!(next.client, Some(5));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(6), 3);
        assert_eq!(next.client, Some(6));
        let next = find_next_client(Direction::Down, false, &hypr_data, current(7), 3);
        assert_eq!(next.client, Some(7));

        // Test up direction without wrapping
        let next = find_next_client(Direction::Up, false, &hypr_data, current(0), 3);
        assert_eq!(next.client, Some(0));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(1), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(2), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(3), 3);
        assert_eq!(next.client, Some(0));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(4), 3);
        assert_eq!(next.client, Some(1));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(5), 3);
        assert_eq!(next.client, Some(2));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(6), 3);
        assert_eq!(next.client, Some(3));
        let next = find_next_client(Direction::Up, false, &hypr_data, current(7), 3);
        assert_eq!(next.client, Some(4));
    }
}
