mod build;
mod configure;
mod extract;
mod test;

use anyhow::Context;
use tracing::{debug_span, trace};

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");
pub const PLUGIN_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const PLUGIN_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLUGIN_OUTPUT_PATH: &str = "/tmp/hyprshell.so";

static ASSET_ZIP_52: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/52/plugin.zip"));
static ASSET_ZIP_54: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/54/plugin.zip"));

pub use configure::PluginConfig;

pub fn generate(config: &PluginConfig, version: &semver::Version) -> anyhow::Result<()> {
    let _span = debug_span!("generate").entered();

    trace!("extracting plugin from zip");
    let dir = extract::extract_plugin(version).context("Failed to extract plugin")?;
    trace!("configuring defs file");
    configure::configure(&dir, config, version).context("unable to configure defs file")?;
    trace!("building plugin");
    build::build(&dir).context("Failed to build plugin")?;
    Ok(())
}
