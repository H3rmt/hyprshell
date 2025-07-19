use anyhow::Context;

mod check;
mod m1t2;
mod migrate_config;

pub use check::check_migration_needed;
pub use migrate_config::migrate;
