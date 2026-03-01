use anyhow::Context;
use config_lib::Config;
use core_lib::{WarnWithDetails, notify_warn};
use exec_lib::binds::{apply_exec_bind, apply_layerrules};
use std::env;
use std::path::Path;
use tracing::{debug_span, info, warn};

pub fn configure_wm_initial(cache_dir: &Path) -> bool {
    exec_lib::reload_hyprland_config()
        .context("Failed to reload hyprland config")
        .warn_details("unable to reload hyprland config");

    if let Err(err) = exec_lib::plugin::load_plugin(cache_dir) {
        notify_warn(
            "Unable to load hyprshell-hyprland plugin, please create a issue on github including the error.\nRun `hyprshell run -vv` to see the logs",
        );
        warn!("Failed to load hyprland plugin: {err:?}");
        return false;
    }
    true
}

pub fn configure_wm(_config: &Config, _cache_dir: &Path) -> anyhow::Result<()> {
    apply_layerrules().warn_details("Failed to apply layerrules");
    Ok(())
}
