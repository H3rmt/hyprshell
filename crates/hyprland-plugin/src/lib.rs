mod build;
mod extract;
mod test;

use anyhow::Context;

pub const PLUGIN_NAME: &str = "hyprshell plugin";
pub const PLUGIN_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const PLUGIN_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

static ASSET_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.zip"));

pub use {build::build_plugin, extract::extract_plugin};
