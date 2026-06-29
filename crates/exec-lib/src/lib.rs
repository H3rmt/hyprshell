pub mod collect;
pub mod listener;
pub mod switch;
mod util;
#[cfg(feature = "live_windows")]
pub mod wayland_capture;

pub mod binds;
pub mod kill;
pub mod run;

pub use util::{
    check_version, get_initial_active, reload_hyprland_config, reset_no_follow_mouse,
    set_follow_mouse_default, set_no_follow_mouse,
};
