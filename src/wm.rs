use anyhow::Context;
use config_lib::Config;
use core_lib::WarnWithDetails;
use exec_lib::binds::{apply_layerrules, reconcile_exec_binds};
use std::path::Path;
use tracing::debug;

pub fn configure_wm(config: &Config, cache_dir: &Path) -> anyhow::Result<()> {
    apply_binds(config, cache_dir).context("Failed to apply binds")?;
    apply_layerrules().warn_details("Failed to apply layerrules");
    debug!("applied layerrules");
    Ok(())
}

fn apply_binds(config: &Config, cache_dir: &Path) -> anyhow::Result<()> {
    let binds = config
        .windows
        .as_ref()
        .map(windows_lib::generate_open_keybinds)
        .unwrap_or_default();
    reconcile_exec_binds(&binds, cache_dir)
}
