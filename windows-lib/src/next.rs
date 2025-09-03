use core_lib::transfer::Direction;
use core_lib::{
    Active, ClientData, ClientId, GetFirstOrLast, HyprlandData, RevIf, WarnWithDetails,
    WorkspaceData, WorkspaceId,
};
use tracing::{debug, debug_span, trace};

pub fn find_next(
    direction: &Direction,
    workspace: bool,
    wrap_workspaces: bool,
    hypr_data: &HyprlandData,
    active: Active,
    workspaces_per_row: usize,
) -> Active {
    let _span =
        debug_span!("find_next", direction =? direction, workspace, active =? active).entered();
    let reverse = matches!(direction, Direction::Left | Direction::Up);
    let next = if workspace {
        if wrap_workspaces {
            find_next_workspace_wrap(reverse, &hypr_data.workspaces, active.workspace)
        } else {
            find_next_workspace(
                direction,
                &hypr_data.workspaces,
                active.workspace,
                workspaces_per_row,
            )
        }
        .unwrap_or(Active {
            client: None,
            workspace: active.workspace,
            monitor: active.monitor,
        })
    } else if let Some(active_client) = active.client {
        find_next_client_wrap(reverse, &hypr_data.clients, active_client).unwrap_or(active)
    } else {
        find_first_client(direction, &hypr_data.clients, &hypr_data.workspaces, active)
            .unwrap_or(active)
    };
    trace!("Next active: {:?}", next);
    next
}

pub fn find_first_client(
    direction: &Direction,
    clients: &[(ClientId, ClientData)],
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: Active,
) -> Option<Active> {
    let get_last = matches!(direction, Direction::Left | Direction::Up);
    clients
        .iter()
        .filter(|(_, c)| c.workspace == active.workspace && c.enabled)
        .get_first_or_last(get_last)
        .or_else(|| {
            trace!("No client found in current workspace, looking for next client in workspaces");
            workspaces
                .iter()
                .rev_if(get_last)
                .skip_while(|(id, _)| id != &active.workspace)
                .find_map(|(id, _)| {
                    trace!("Finding first client in workspace {id:?}");
                    clients
                        .iter()
                        .filter(|(_, c)| c.workspace == *id && c.enabled)
                        .get_first_or_last(get_last)
                })
                .or_else(|| {
                    clients
                        .iter()
                        .filter(|(_, c)| c.monitor == active.monitor)
                        .get_first_or_last(get_last)
                        .or_else(|| clients.iter().get_first_or_last(get_last))
                })
        })
        .map(|(id, data)| Active {
            client: Some(*id),
            workspace: data.workspace,
            monitor: data.monitor,
        })
}

pub fn find_next_client_wrap(
    reverse: bool,
    clients: &[(ClientId, ClientData)],
    active: ClientId,
) -> Option<Active> {
    if clients.is_empty() {
        debug!("No clients available, returning None");
        return None;
    }

    let offset = (if reverse { -1 } else { 1 } as isize).rem_euclid(
        isize::try_from(clients.iter().filter(|(_, c)| c.enabled).count())
            .warn_details("unable convert clients len")?,
    ) as usize;
    trace!("Finding next client wrap with offset: {}", offset);

    clients
        .iter()
        .cycle()
        .skip_while(|(id, _)| *id != active) // skip until current element
        .filter(|(_, c)| c.enabled)
        .nth(offset)
        .map(|(id, data)| Active {
            client: Some(*id),
            workspace: data.workspace,
            monitor: data.monitor,
        })
}

pub fn find_next_workspace(
    direction: &Direction,
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: WorkspaceId,
    workspaces_per_row: usize,
) -> Option<Active> {
    if workspaces.is_empty() {
        debug!("No workspaces available, returning None");
        return None;
    }

    let offset = match direction {
        Direction::Right => 1,
        Direction::Left => -1,
        Direction::Up => -isize::try_from(workspaces_per_row)
            .warn_details("unable convert workspaces_per_row")?,
        Direction::Down => {
            isize::try_from(workspaces_per_row).warn_details("unable convert workspaces_per_row")?
        }
    };
    trace!("Finding next workspace with offset: {}", offset);

    let current = workspaces.iter().position(|w| w.0 == active).unwrap_or(0);
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    workspaces
        .iter()
        .filter(|(_, c)| c.any_client_enabled)
        .nth(
            (current as isize + offset)
                .max(0)
                .min((workspaces.len() - 1) as isize) as usize,
        )
        .map(|(id, data)| Active {
            client: None,
            workspace: *id,
            monitor: data.monitor,
        })
}

pub fn find_next_workspace_wrap(
    reverse: bool,
    workspaces: &[(WorkspaceId, WorkspaceData)],
    active: WorkspaceId,
) -> Option<Active> {
    if workspaces.is_empty() {
        debug!("No workspaces available, returning None");
        return None;
    }
    #[allow(clippy::cast_possible_wrap)]
    let offset =
        (if reverse { -1 } else { 1 } as isize).rem_euclid(workspaces.len() as isize) as usize;
    trace!("Finding next workspace wrap with offset: {}", offset);

    workspaces
        .iter()
        .cycle()
        .skip_while(|(id, _)| *id != active) // skip until current element
        .filter(|(_, c)| c.any_client_enabled)
        .nth(offset)
        .map(|(id, data)| Active {
            client: None,
            workspace: *id,
            monitor: data.monitor,
        })
}

// TODO add some tests here
