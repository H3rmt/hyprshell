mod components;
mod start;
mod structs;
mod util;

pub const APPLICATION_EDIT_ID: &str = "com.github.h3rmt.hyprshell-edit";
pub use start::start;

#[allow(clippy::wildcard_imports)]
pub(crate) use structs::*;
