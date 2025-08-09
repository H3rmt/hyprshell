#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

mod css;
mod data;
mod desktop_map;
mod global;
mod icon;
mod keybinds;
mod next;
mod overview;
mod sort;
mod switch;

pub use css::get_css;
pub use desktop_map::{get_icon_name_by_name_from_desktop_files, reload_class_to_icon_map};
pub use global::{WindowsOverviewData, WindowsSwitchData};
pub use keybinds::generate_open_keybinds;
pub use overview::{
    close_overview, create_windows_overview_window, open_overview, overview_already_hidden,
    overview_already_open, stop_overview, update_overview,
};
pub use switch::{
    close_switch, create_windows_switch_window, open_switch, stop_switch, switch_already_hidden,
    switch_already_open, update_switch,
};
