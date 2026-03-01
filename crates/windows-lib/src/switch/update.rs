use crate::data::{SortConfig, collect_data};
use crate::global::WindowsSwitchData;
use crate::next::{find_next_client, find_next_workspace};
use crate::switch::render_switch;
use anyhow::Context;
use core_lib::transfer::SwitchSwitchConfig;
use relm4::adw::prelude::WidgetExt;
use tracing::{debug_span, instrument};

pub fn switch_to_next(data: &mut WindowsSwitchData, config: &SwitchSwitchConfig) {
    let _span = debug_span!("switch_to_next").entered();

    // let active = if data.config.switch_workspaces {
    //     find_next_workspace(
    //         &config.direction,
    //         true,
    //         &data.hypr_data,
    //         data.active,
    //         data.config.items_per_row,
    //     )
    // } else {
    //     find_next_client(
    //         &config.direction,
    //         true,
    //         &data.hypr_data,
    //         data.active,
    //         data.config.items_per_row,
    //     )
    // };
    // data.active = active;

    if data.config.switch_workspaces {
        for button in data.clients.values() {
            button.remove_css_class("active");
        }
        // for (id, button) in &data.workspaces {
        //     button.remove_css_class("active");
        //     if active.workspace == *id {
        //         button.add_css_class("active");
        //     }
        // }
    } else {
        for button in data.workspaces.values() {
            button.remove_css_class("active");
        }
        // for (id, button) in &data.clients {
        //     button.remove_css_class("active");
        //     if active.client == Some(*id) {
        //         button.add_css_class("active");
        //     }
        // }
    }
}

#[instrument(level = "debug", skip_all)]
pub fn update_data(data: &mut WindowsSwitchData) -> anyhow::Result<()> {
    let (clients_data, _) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: true,
        exclude_workspaces: data.config.exclude_workspaces.clone(),
    })
    .context("Failed to collect data")?;

    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;
    render_switch(data, clients_data, data.active, &remove_html)
}
