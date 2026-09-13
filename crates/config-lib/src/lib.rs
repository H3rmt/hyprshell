mod check;
mod edit_sync;
mod explain;
mod io;
#[cfg(not(feature = "disable_migrations"))]
mod migrate;
mod modifier;
mod structs;
pub mod style;
pub mod workarrounds;

pub use check::check;
pub use explain::explain;
pub use io::load_and_migrate_config;
pub use io::save::write_io_config;
pub use modifier::*;
pub use structs::*;

pub const CURRENT_CONFIG_VERSION: u64 = 5;
pub const CURRENT_WORKARROUND_VERSION: u64 = 1;
