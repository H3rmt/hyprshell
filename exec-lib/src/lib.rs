#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]
pub mod collect;
pub mod listener;
pub mod switch;
mod util;

pub mod binds;
pub mod plugin;
pub mod run;

pub use util::*;
