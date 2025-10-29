use crate::global::WindowsSwitchData;
use crate::next::find_next;
use adw::gtk::prelude::*;
use core_lib::transfer::{Direction, SwitchSwitchConfig};
use tracing::debug_span;

pub fn update_switch(data: &mut WindowsSwitchData, config: &SwitchSwitchConfig) {
    let _span = debug_span!("update_switch").entered();

    let active = find_next(
        &if config.reverse {
            Direction::Left
        } else {
            Direction::Right
        },
        data.config.switch_workspaces,
        true,
        &data.hypr_data,
        data.active,
        0,
    );
    data.active = active;

    if data.config.switch_workspaces {
        for button in data.clients.values() {
            button.remove_css_class("active");
        }
        for (id, button) in &data.workspaces {
            button.remove_css_class("active");
            if active.workspace == *id {
                button.add_css_class("active");
            }
        }
    } else {
        for button in data.workspaces.values() {
            button.remove_css_class("active");
        }
        for (id, button) in &data.clients {
            button.remove_css_class("active");
            if active.client == Some(*id) {
                button.add_css_class("active");
            }
        }
    }
}
