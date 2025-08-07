use anyhow::Context;
use config_lib::Config;
use core_lib::WarnWithDetails;
use core_lib::binds::generate_bind_kill;
use exec_lib::binds::{apply_exec_bind, apply_layerrules};
use std::env;
use tracing::{Level, debug_span, info, span};

pub fn create_binds(config: &Config) -> anyhow::Result<()> {
    let _span = debug_span!("create_binds").entered();
    generate_bind_kill(&config.kill_bind)
        .and_then(|bind| apply_exec_bind(&bind))
        .context("Failed to apply kill bind")?;

    if let Some(windows) = &config.windows {
        if env::var_os("HYPRSHELL_NO_USE_PLUGIN").is_none() {
            if let Some(switch) = windows.switch.as_ref() {
                exec_lib::plugin::load_plugin(switch.modifier)
                    .warn_details("Failed to load plugin for switch modifier");
                info!("Loaded hyprland plugin")
            }
        }
        for bind in windows_lib::generate_open_keybinds(windows) {
            apply_exec_bind(&bind).context("Failed to apply open keybinds for windows")?;
        }
    };

    if config.layerrules {
        apply_layerrules().warn_details("Failed to apply layerrules");
    }
    Ok(())
}
