mod actions;
mod applications;
#[cfg(feature = "calc")]
mod calc;
mod main;
mod path;
mod search;
mod shell;
mod terminal;

pub use applications::get_stored_runs as get_applications_stored_runs;
pub use applications::reload_desktop_entries_map as reload_applications_desktop_entries_map;

#[cfg(feature = "calc")]
pub use calc::init_context as init_calc_context;
#[cfg(not(feature = "calc"))]
pub const fn init_calc_context() {}

pub use main::{
    HighlightedText, LaunchChildItem, LaunchItem, MatchedLaunchItem, PluginReturn,
    StaticLaunchItem, TextSpan, get_child_launch_items, get_input_driven_launch_items,
    get_launch_items, get_static_launch_items, get_static_options_chars, launch,
};
