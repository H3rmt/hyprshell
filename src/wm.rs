use anyhow::Context;
use config_lib::Config;
use core_lib::WarnWithDetails;
use exec_lib::binds::{apply_exec_bind, apply_layerrules};
use tracing::debug;

pub fn configure_wm_initial() {
    exec_lib::reload_hyprland_config()
        .context("Failed to reload hyprland config")
        .warn_details("unable to reload hyprland config");
}

pub fn configure_wm(config: &Config) -> anyhow::Result<()> {
    apply_layerrules().warn_details("Failed to apply layerrules");
    debug!("applied layerrules");
    apply_binds(config).context("Failed to apply binds")?;
    Ok(())
}

fn apply_binds(config: &Config) -> anyhow::Result<()> {
    if let Some(windows) = &config.windows {
        for bind in windows_lib::generate_open_keybinds(windows) {
            apply_exec_bind(&bind).context("Failed to apply open keybinds for windows")?;
        }
    }
    Ok(())
}
