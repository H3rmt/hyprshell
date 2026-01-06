use anyhow::Context;
use config_lib::Config;
use core_lib::{WarnWithDetails, notify_warn};
use exec_lib::binds::{apply_exec_bind, apply_layerrules};
use std::env;
use tracing::{debug_span, info, warn};

pub fn configure_wm(config: &Config) -> anyhow::Result<()> {
    let _span = debug_span!("create_binds").entered();

    if env::var_os("HYPRSHELL_NO_USE_PLUGIN").is_none() {
        if let Err(err) = plugin(config) {
            notify_warn(
                "Unable to load hyprland plugin, please create a issue on github including the error. pass -vv to see the logs",
            );
            warn!("Failed to load hyprland plugin: {err:?}");
            info!("Falling back to default keybinds");
            apply_binds(config)?;
        }
    } else {
        apply_binds(config)?;
    }

    apply_layerrules().warn_details("Failed to apply layerrules");
    Ok(())
}

fn plugin(config: &Config) -> anyhow::Result<()> {
    if let Some(windows) = &config.windows {
        let switch = windows.switch.as_ref().map(|s| (s.modifier, s.key.clone()));
        let overview = windows
            .overview
            .as_ref()
            .map(|o| (o.modifier, o.key.clone()));
        exec_lib::plugin::load_plugin(switch, overview)
            .context("Failed to load hyprland plugin")?;
    }
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
