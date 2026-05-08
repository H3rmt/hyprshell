use crate::data::{SortConfig, collect_data};
use crate::global::WindowsOverviewData;
use crate::overview::render::render_overview;
use anyhow::Context;
use async_channel::Sender;
use core_lib::WarnWithDetails;
use core_lib::transfer::ExternalTransferType;
use relm4::adw::gtk::prelude::*;
use tracing::debug_span;

#[must_use]
pub fn overview_already_open(data: &WindowsOverviewData) -> bool {
    data.window_list.iter().any(|w| w.0.get_visible())
}

#[allow(clippy::too_many_lines)]
pub fn open_overview(
    data: &mut WindowsOverviewData,
    event_sender: &Sender<ExternalTransferType>,
) -> anyhow::Result<()> {
    let _span = debug_span!("open_overview").entered();

    let (hypr_data, active) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: false,
        exclude_workspaces: data.config.exclude_workspaces.clone(),
    })
    .context("Failed to collect data")?;
    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    render_overview(data, hypr_data, active, &remove_html, event_sender);
    Ok(())
}
