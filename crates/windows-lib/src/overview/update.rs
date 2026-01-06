use crate::global::WindowsOverviewData;
use crate::next::{find_next_client, find_next_workspace};
use core_lib::transfer::{Direction, SwitchOverviewConfig};
use relm4::adw::gtk::prelude::*;
use tracing::{debug_span, error};

pub fn update_overview(data: &mut WindowsOverviewData, config: &SwitchOverviewConfig) {
    let _span = debug_span!("update_overview").entered();

    let active = if config.workspace {
        find_next_workspace(
            &config.direction,
            false,
            &data.hypr_data,
            data.active,
            data.config.items_per_row,
        )
    } else {
        if config.direction == Direction::Up || config.direction == Direction::Down {
            error!(
                "Clients in overview can only be switched left and right (forwards and backwards)"
            );
            return;
        }
        find_next_client(
            &config.direction,
            false,
            &data.hypr_data,
            data.active,
            data.config.items_per_row,
        )
    };
    data.active = active;

    for monitor_data in data.window_list.values_mut() {
        if config.workspace {
            for button in monitor_data.clients.values_mut() {
                button.remove_css_class("active");
            }
            for (id, button) in &mut monitor_data.workspaces {
                button.remove_css_class("active");
                if active.workspace == *id {
                    button.add_css_class("active");
                }
            }
        } else {
            for button in monitor_data.workspaces.values_mut() {
                button.remove_css_class("active");
            }
            for (id, button) in &mut monitor_data.clients {
                button.remove_css_class("active");
                if active.client == Some(*id) {
                    button.add_css_class("active");
                }
            }
        }
    }
}
