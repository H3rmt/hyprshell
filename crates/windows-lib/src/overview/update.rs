use crate::data::{SortConfig, collect_data};
use crate::global::WindowsOverviewData;
use crate::next::{find_next_client, find_next_workspace};
use crate::overview::render::render_overview;
use anyhow::Context;
use async_channel::Sender;
use core_lib::transfer::{Direction, ExternalTransferType, SwitchOverviewConfig};
use relm4::adw::gtk::prelude::*;
use tracing::{debug_span, error, instrument};

pub fn switch_to_next(data: &mut WindowsOverviewData, config: &SwitchOverviewConfig) {
    let _span = debug_span!("switch_to_next").entered();

    // let active = if config.workspace {
    // find_next_workspace(
    //     &config.direction,
    //     false,
    //     &data.hypr_data,
    //     data.active,
    //     data.config.items_per_row,
    // )
    // } else {
    //     if config.direction == Direction::Up || config.direction == Direction::Down {
    //         error!(
    //             "Clients in overview can only be switched left and right (forwards and backwards)"
    //         );
    //         return;
    //     }
    //     find_next_client(
    //         &config.direction,
    //         false,
    //         &data.hypr_data,
    //         data.active,
    //         data.config.items_per_row,
    //     )
    // };
    // data.active = active;

    for monitor_data in data.window_list.values_mut() {
        if config.workspace {
            for button in monitor_data.clients.values_mut() {
                button.remove_css_class("active");
            }
            // for (id, button) in &mut monitor_data.workspaces {
            //     button.remove_css_class("active");
            //     if active.workspace == *id {
            //         button.add_css_class("active");
            //     }
            // }
        } else {
            for button in monitor_data.workspaces.values_mut() {
                button.remove_css_class("active");
            }
            // for (id, button) in &mut monitor_data.clients {
            //     button.remove_css_class("active");
            //     if active.client == Some(*id) {
            //         button.add_css_class("active");
            //     }
            // }
        }
    }
}

#[instrument(level = "debug", skip_all)]
pub fn update_data(
    data: &mut WindowsOverviewData,
    event_sender: &Sender<ExternalTransferType>,
) -> anyhow::Result<()> {
    let (hypr_data, _) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: false,
        exclude_workspaces: data.config.exclude_workspaces.clone(),
    })
    .context("Failed to collect data")?;

    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;
    render_overview(data, hypr_data, data.active, &remove_html, event_sender);
    Ok(())
}
