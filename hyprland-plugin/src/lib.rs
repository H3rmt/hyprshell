mod build;
mod configure;
mod extract;

use anyhow::Context;
use std::io::Read;
use std::path::Path;

static ASSET_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.zip"));

pub fn generate() -> anyhow::Result<Box<Path>> {
    let path = extract::extract_plugin().context("Failed to extract plugin")?;
    configure::configure(&path).context("unable to configure defs file")?;
    let out = build::build(&path).context("Failed to build plugin")?;
    Ok(out.into_boxed_path())
}
