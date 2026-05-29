mod css;
pub mod debug;
mod plugins;
mod plugins_boxes;
mod result;
mod root;

pub use css::get_css;
pub use plugins::{get_applications_stored_runs, reload_applications_desktop_entries_map};

pub use root::*;
