use core_lib::{ClientData, ClientId, MonitorId, WorkspaceData, WorkspaceId};
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Sorts clients with complex sorting
///
/// * 'clients' - Vector of clients to sort
/// * '`ignore_workspaces`' - Don't split clients into workspaces (treat all clients on monitor as one workspace)
/// * '`ignore_monitors`' - Don't split clients into monitors (treat all clients as one monitor)
pub fn sort_clients_by_position(
    clients: Vec<(ClientId, ClientData)>,
) -> Vec<(ClientId, ClientData)> {
    // monitor -> workspace -> clients
    let monitors: Vec<Vec<Vec<(ClientId, ClientData)>>> = {
        let mut monitors: BTreeMap<MonitorId, BTreeMap<WorkspaceId, Vec<(ClientId, ClientData)>>> =
            BTreeMap::new();
        for (addr, client) in clients {
            monitors
                .entry(client.monitor)
                .or_default()
                .entry(client.workspace)
                .or_default()
                .push((addr, client));
        }
        monitors
            .into_values()
            .map(|m| m.into_values().collect())
            .collect()
    };

    let mut sorted_clients = Vec::new();

    for workspaces in monitors {
        for mut clients in workspaces {
            clients.sort_by(|(_, a), (_, b)| {
                if a.x == b.x {
                    a.y.cmp(&b.y)
                } else {
                    a.x.cmp(&b.x)
                }
            });
            let mut queue: VecDeque<(ClientId, ClientData)> = VecDeque::from(clients);

            let mut line_start = queue.pop_front();
            while let Some((current_addr, current)) = line_start {
                let mut current_bottom = current.y + current.height;
                sorted_clients.push((current_addr, current));

                loop {
                    let mut next_index = None;

                    /*
                    1. Check If Top left of window is higher or lower than bottom left of current
                    2. Check if any window(not taken) on left top is higher or lower than current Lower (if true take this)
                    3. Check if any window(not taken) on left bottom is higher than current bottom (if true take this)
                    => Take if Top higher than current Bottom and no window on left has higher Top than window Bottom
                     */
                    for (i, (_, client)) in queue.iter().enumerate() {
                        let client_top = client.y;
                        let client_bottom = client.y + client.height;
                        let client_left = client.x;

                        if client_top < current_bottom {
                            // 1.
                            // client top is inside current row

                            // 2.
                            let on_left = queue
                                .iter()
                                .enumerate()
                                .find(|(_, (_, c))| c.x < client_left && c.y < client_bottom);

                            // 3.
                            let on_left_2 = queue.iter().enumerate().find(|(_, (_, c))| {
                                c.x < client_left && c.y + c.height < client_bottom
                            });

                            match (on_left, on_left_2) {
                                (Some((idx, (_, c))), _) | (_, Some((idx, (_, c)))) => {
                                    current_bottom = c.y + c.height;
                                    next_index = Some(idx);
                                }
                                (None, None) => {
                                    next_index = Some(i);
                                }
                            }
                            break;
                        }
                    }
                    match next_index.and_then(|i| queue.remove(i)) {
                        Some(next) => {
                            sorted_clients.push(next);
                        }
                        None => {
                            break;
                        }
                    }
                }
                line_start = queue.pop_front();
            }
        }
    }

    sorted_clients
}

pub fn sort_clients_by_recent(clients: &mut [(ClientId, ClientData)]) {
    let focus_map = clients
        .iter()
        .map(|(id, client_data)| (*id, client_data.focus_history_id))
        .collect::<HashMap<ClientId, i8>>();
    clients.sort_by(|(a_addr, a), (b_addr, b)| {
        match (focus_map.get(a_addr), focus_map.get(b_addr)) {
            (None, None) => a.focus_history_id.cmp(&b.focus_history_id), // both none -> sort by focus_history_id
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a_id), Some(b_id)) => a_id.cmp(b_id),
        }
    });
}

pub fn sort_workspaces_by_recent(
    workspaces: &mut [(WorkspaceId, WorkspaceData)],
    clients: &[(ClientId, ClientData)],
) {
    let mut ordering = vec![];
    for (_, client) in clients {
        if !ordering.contains(&client.workspace) {
            ordering.push(client.workspace);
        }
    }

    workspaces.sort_by(|(a_id, _), (b_id, _)| {
        let a_pos = ordering
            .iter()
            .position(|id| id == a_id)
            .unwrap_or(usize::MAX);
        let b_pos = ordering
            .iter()
            .position(|id| id == b_id)
            .unwrap_or(usize::MAX);
        a_pos.cmp(&b_pos)
    });
}

pub fn sort_workspaces_by_position(workspaces: &mut [(WorkspaceId, WorkspaceData)]) {
    workspaces.sort_by(|(_, a), (_, b)| a.x.cmp(&b.x));
}
