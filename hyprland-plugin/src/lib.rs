mod build;
mod configure;
mod extract;

use anyhow::Context;

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");
pub const PLUGIN_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const PLUGIN_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLUGIN_OUTPUT_PATH: &str = "/tmp/hyprshell.so";

static ASSET_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.zip"));

pub fn generate() -> anyhow::Result<()> {
    let dir = extract::extract_plugin().context("Failed to extract plugin")?;
    configure::configure(&dir).context("unable to configure defs file")?;
    build::build(&dir).context("Failed to build plugin")?;
    Ok(())
}
