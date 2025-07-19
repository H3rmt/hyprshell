mod check;
mod explain;
#[cfg(feature = "generate_config")]
pub mod generate;
mod load;
mod migrate;
mod modifier;
mod save;
mod structs;

pub use check::check;
pub use explain::explain;
pub use load::load_and_migrate_config;
pub use modifier::*;
pub use save::write_config;
pub use structs::*;

pub(crate) const CURRENT_CONFIG_VERSION: u16 = 2;
