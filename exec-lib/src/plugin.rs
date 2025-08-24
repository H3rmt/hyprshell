use anyhow::Context;
use config_lib::Modifier;
use hyprland::ctl::plugin;
use hyprland_plugin::PluginConfig;
use std::path::Path;
use tracing::{debug, debug_span, trace};

pub fn load_plugin(
    switch: Option<Modifier>,
    overview: Option<(Modifier, Box<str>)>,
) -> anyhow::Result<()> {
    let _span = debug_span!("load_plugin").entered();
    let config = PluginConfig {
        xkb_key_switch_mod: switch.map(|s| Box::from(mod_to_xkb_key(s))),
        xkb_key_overview_mod: overview
            .as_ref()
            .map(|(r#mod, _)| Box::from(r#mod.to_string())),
        xkb_key_overview_key: overview.map(|(_, key)| key),
    };

    if check_new_plugin_needed(&config) {
        unload().context("unable to unload old plugin")?;
        hyprland_plugin::generate(&config).context("unable to generate plugin")?;
        trace!(
            "generated plugin at {:?}",
            hyprland_plugin::PLUGIN_OUTPUT_PATH
        );
        plugin::load(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH))
            .context("unable to load plugin")?;
        trace!("loaded plugin");
    } else {
        debug!("plugin already loaded, skipping");
    }

    Ok(())
}

pub fn check_new_plugin_needed(config: &PluginConfig) -> bool {
    let plugins = plugin::list().unwrap_or_default();
    trace!("plugins: {:?}", plugins);
    for plugin in plugins {
        if plugin.name == hyprland_plugin::PLUGIN_NAME {
            let Some(desc) = plugin.description.split(" - ").last() else {
                continue;
            };
            if desc == config.to_string() {
                // config didn't change, no need to reload
                return false;
            }
        }
    }
    true
}

pub fn unload() -> anyhow::Result<()> {
    let plugins = plugin::list().unwrap_or_default();
    for plugin in plugins {
        if plugin.name == hyprland_plugin::PLUGIN_NAME {
            debug!("plugin loaded, unloading it");
            plugin::unload(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH)).with_context(|| {
                format!(
                    "unable to unload old plugin at: {}",
                    hyprland_plugin::PLUGIN_OUTPUT_PATH
                )
            })?;
            debug!("plugin unloaded");
        }
    }
    Ok(())
}

pub const fn mod_to_xkb_key(r#mod: Modifier) -> &'static str {
    match r#mod {
        Modifier::Alt => "XKB_KEY_Alt",
        Modifier::Ctrl => "XKB_KEY_Control",
        Modifier::Super => "XKB_KEY_Super",
    }
}
