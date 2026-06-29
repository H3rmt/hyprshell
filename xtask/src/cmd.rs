use std::process;

use anyhow::Context;
use tracing::{debug, info};

pub fn run_cargo_command(args: &[&str], dry: bool) -> anyhow::Result<u32> {
    if dry {
        info!("Dry run: would run cargo with args: {args:?}");
        return Ok(0);
    }
    debug!("Running cargo with args: {args:?}");
    let status = process::Command::new("cargo")
        .args(args)
        .status()
        .context("Failed to run cargo clippy")?;
    if !status.success() {
        debug!("Cargo command failed with status: {status}");
    }
    Ok(status.code().unwrap_or(1).cast_unsigned())
}
