pub mod actions;
mod check;
mod explain;
mod load;
mod migrate;
mod modifier;
mod save;
mod structs;
pub mod style;

pub use check::check;
pub use explain::explain;
pub use load::load_and_migrate_config;
pub use modifier::*;
pub use save::write_config;
pub use structs::*;

pub const CURRENT_CONFIG_VERSION: u16 = 3;
