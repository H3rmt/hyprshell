use crate::data::{SortConfig, collect_data};
use crate::global::WindowsSwitchData;
use crate::next::{find_next_client, find_next_workspace};
use anyhow::Context;
use core_lib::WarnWithDetails;
use core_lib::transfer::{Direction, OpenSwitch};
use core_lib::{ClientData, ClientId, WarnWithDetails};
use exec_lib::{get_current_monitor, set_no_follow_mouse};
use relm4::adw::gtk::gdk::Cursor;
use relm4::adw::gtk::prelude::*;
use relm4::adw::gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use std::borrow::Cow;
use tracing::{debug_span, trace};

#[must_use]
pub fn switch_already_open(data: &WindowsSwitchData) -> bool {
    data.window.get_visible()
}

pub fn open_switch(data: &mut WindowsSwitchData, config: &OpenSwitch) -> anyhow::Result<()> {
    let _span = debug_span!("open_switch").entered();
    set_no_follow_mouse().warn_details("Failed to set set_remain_focused");

    let (clients_data, active_prev) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: true,
    })
    .context("Failed to collect data")?;
    let dir = if config.reverse {
        Direction::Left
    } else {
        Direction::Right
    };
    let active = if data.config.switch_workspaces {
        find_next_workspace(
            &dir,
            true,
            &clients_data,
            active_prev,
            data.config.items_per_row,
        )
    } else {
        find_next_client(
            &dir,
            true,
            &clients_data,
            active_prev,
            data.config.items_per_row,
        )
    };

    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    trace!("Showing window {:?}", data.window.id());
    data.window.set_visible(true);

    render_switch(data, clients_data, active, &remove_html)
}
