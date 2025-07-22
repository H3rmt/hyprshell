use anyhow::Context;
use config_lib::Modifier;
use hyprland::ctl::plugin;
use hyprland_plugin::PluginConfig;
use std::path::Path;
use tracing::{Level, debug, span, trace};

pub fn load_plugin(modifier: Modifier) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "load_plugin").entered();

    let plugins = plugin::list().unwrap_or_default();
    trace!("plugins: {:?}", plugins);
    for plugin in plugins {
        if plugin.name == hyprland_plugin::PLUGIN_NAME {
            debug!("plugin already loaded, unloading it");
            plugin::unload(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH)).with_context(|| {
                format!(
                    "unable to unload old plugin at: {}",
                    hyprland_plugin::PLUGIN_OUTPUT_PATH
                )
            })?;
            debug!("plugin unloaded");
        }
    }

    hyprland_plugin::generate(&PluginConfig {
        switch_mod: mod_to_hyprland_keycode(modifier).to_string(),
    })
    .context("unable to generate plugin: {err:?}")?;
    trace!(
        "generated plugin at {:?}",
        hyprland_plugin::PLUGIN_OUTPUT_PATH
    );
    plugin::load(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH))
        .context("unable to load plugin")?;
    trace!("loaded plugin");
    Ok(())
}

pub fn mod_to_hyprland_keycode(r#mod: Modifier) -> u32 {
    match r#mod {
        Modifier::Alt => 56,
        Modifier::Ctrl => 29,
        Modifier::Super => 125,
        Modifier::Shift => 42,
    }
}
