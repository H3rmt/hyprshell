use crate::WindowsGlobal;
use crate::next::find_next;
use core_lib::transfer::SwitchConfig;
use gtk::prelude::*;
use tracing::{Level, span};

pub fn update_overview(global: &WindowsGlobal, config: SwitchConfig) {
    let _span = span!(Level::TRACE, "update_overview").entered();

    let mut data = global.data.borrow_mut();
    let active = find_next(
        &config.direction,
        config.workspace,
        &data.hypr_data,
        data.active,
        global.workspaces_per_row as usize,
    );
    data.active = active;

    for monitor_data in data.monitor_list.values_mut() {
        if config.workspace {
            for (_, overlay) in monitor_data.client_refs.iter_mut() {
                overlay.remove_css_class("active");
            }
            for (id, overlay) in monitor_data.workspace_refs.iter_mut() {
                overlay.remove_css_class("active");
                if active.workspace == *id {
                    overlay.add_css_class("active");
                }
            }
        } else {
            for (_, overlay) in monitor_data.workspace_refs.iter_mut() {
                overlay.remove_css_class("active");
            }
            for (id, overlay) in monitor_data.client_refs.iter_mut() {
                overlay.remove_css_class("active");
                if active.client == Some(*id) {
                    overlay.add_css_class("active");
                }
            }
        }
    }
}
