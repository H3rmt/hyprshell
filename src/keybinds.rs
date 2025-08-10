use anyhow::Context;
use config_lib::Config;
use core_lib::WarnWithDetails;
use core_lib::binds::generate_bind_kill;
use exec_lib::binds::{apply_exec_bind, apply_layerrules};
use std::env;
use tracing::{debug_span, info};

pub fn create_binds(config: &Config) -> anyhow::Result<()> {
    let _span = debug_span!("create_binds").entered();

    if env::var_os("HYPRSHELL_NO_USE_PLUGIN").is_none() {
        if let Some(windows) = &config.windows {
            let switch = windows.switch.as_ref().map(|s| s.modifier);
            let overview = windows
                .overview
                .as_ref()
                .map(|o| (o.modifier, o.key.clone()));
            exec_lib::plugin::load_plugin(switch, overview)
                .warn_details("Failed to load plugin for switch modifier");
            info!("Loaded hyprland plugin");
        }
    } else {
        if let Some(windows) = &config.windows {
            for bind in windows_lib::generate_open_keybinds(windows) {
                apply_exec_bind(&bind).context("Failed to apply open keybinds for windows")?;
            }
        }

        generate_bind_kill(&config.kill_bind)
            .and_then(|bind| apply_exec_bind(&bind))
            .context("Failed to apply kill bind")?;
    }

    if config.layerrules {
        apply_layerrules().warn_details("Failed to apply layerrules");
    }
    Ok(())
}
