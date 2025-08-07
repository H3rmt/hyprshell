use crate::global::WindowsSwitchData;
use crate::next::find_next;
use core_lib::transfer::{Direction, SwitchSwitchConfig};
use gtk::prelude::*;
use tracing::{Level, debug_span, span};

pub fn update_switch(data: &mut WindowsSwitchData, config: SwitchSwitchConfig) {
    let _span = debug_span!("update_switch").entered();

    let active = find_next(
        &if config.reverse {
            Direction::Left
        } else {
            Direction::Right
        },
        data.config.show_workspaces,
        true,
        &data.hypr_data,
        data.active,
        0,
    );
    data.active = active;

    if data.config.show_workspaces {
        for (_, button) in data.clients.iter_mut() {
            button.remove_css_class("active");
        }
        for (id, button) in data.workspaces.iter_mut() {
            button.remove_css_class("active");
            if active.workspace == *id {
                button.add_css_class("active");
            }
        }
    } else {
        for (_, button) in data.workspaces.iter_mut() {
            button.remove_css_class("active");
        }
        for (id, button) in data.clients.iter_mut() {
            button.remove_css_class("active");
            if active.client == Some(*id) {
                button.add_css_class("active");
            }
        }
    }
}
