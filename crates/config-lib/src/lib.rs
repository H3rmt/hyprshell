mod check;
mod explain;
mod io;
#[cfg(not(feature = "disable_migrations"))]
mod migrate;
mod modifier;
mod structs;
pub mod style;

pub use check::check;
pub use explain::explain;
pub use io::load_and_migrate_config;
pub use io::save::write_config;
pub use modifier::*;
pub use structs::*;

pub const CURRENT_CONFIG_VERSION: u16 = 4;
