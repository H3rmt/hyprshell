use crate::global::WindowsOverviewData;
use crate::next::find_next;
use core_lib::transfer::SwitchOverviewConfig;
use gtk::prelude::*;
use tracing::{Level, debug_span, span};

pub fn update_overview(data: &mut WindowsOverviewData, config: SwitchOverviewConfig) {
    let _span = debug_span!("update_overview").entered();

    let active = find_next(
        &config.direction,
        config.workspace,
        false,
        &data.hypr_data,
        data.active,
        data.config.items_per_row as usize,
    );
    data.active = active;

    for monitor_data in data.window_list.values_mut() {
        if config.workspace {
            for (_, button) in monitor_data.clients.iter_mut() {
                button.remove_css_class("active");
            }
            for (id, button) in monitor_data.workspaces.iter_mut() {
                button.remove_css_class("active");
                if active.workspace == *id {
                    button.add_css_class("active");
                }
            }
        } else {
            for (_, button) in monitor_data.workspaces.iter_mut() {
                button.remove_css_class("active");
            }
            for (id, button) in monitor_data.clients.iter_mut() {
                button.remove_css_class("active");
                if active.client == Some(*id) {
                    button.add_css_class("active");
                }
            }
        }
    }
}
