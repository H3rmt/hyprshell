use core_lib::{ClientData, ClientId, MonitorData, MonitorId, WorkspaceData, WorkspaceId};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Sorts clients with complex sorting
pub fn sort_clients_by_position(
    clients: Vec<(ClientId, ClientData)>,
    workspaces: &Vec<(WorkspaceId, WorkspaceData)>,
    monitors: &Vec<(MonitorId, MonitorData)>,
) -> Vec<(ClientId, ClientData)> {
    // monitor -> workspace -> clients
    let mut presorted_clients = {
        let mut presorted_clients: BTreeMap<
            MonitorId,
            BTreeMap<WorkspaceId, Vec<(ClientId, ClientData)>>,
        > = BTreeMap::new();
        for (addr, client) in clients {
            presorted_clients
                .entry(client.monitor)
                .or_default()
                .entry(client.workspace)
                .or_default()
                .push((addr, client));
        }
        presorted_clients
    };

    let mut sorted_clients = Vec::new();
    for (monitor_id, _) in monitors {
        for (workspace_id, _) in workspaces {
            let mut clients = presorted_clients
                .get_mut(monitor_id)
                .and_then(|m| m.remove(workspace_id))
                .unwrap_or(vec![]);
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
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
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

pub fn sort_monitor_by_x(monitors: &mut [(MonitorId, MonitorData)]) {
    monitors.sort_by_key(|(_, a)| a.x);
}

pub fn sort_workspaces_by_position(
    workspaces: &mut [(WorkspaceId, WorkspaceData)],
    monitors: &[(MonitorId, MonitorData)],
) {
    workspaces.sort_by(|(a_id, a), (b_id, b)| {
        let monitor_a = monitors
            .iter()
            .position(|(id, _)| id == &a.monitor)
            .unwrap_or(0);
        let monitor_b = monitors
            .iter()
            .position(|(id, _)| id == &b.monitor)
            .unwrap_or(0);
        match monitor_a.cmp(&monitor_b) {
            // move special workspaces with -ids after normal workspaces
            Ordering::Equal => {
                // -10 > 10
                if *a_id < 0 && *b_id > 0 {
                    return Ordering::Greater;
                }
                // 10 > -10
                if *a_id > 0 && *b_id < 0 {
                    return Ordering::Less;
                }
                a_id.cmp(b_id)
            }
            other => other,
        }
    });
}
