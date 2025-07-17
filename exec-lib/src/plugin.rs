use anyhow::{Context, bail};
use hyprland::ctl::plugin;
use std::path::Path;
use tracing::{debug, info, trace};

pub fn test() -> anyhow::Result<()> {
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

    hyprland_plugin::generate().context("unable to generate plugin: {err:?}")?;
    trace!(
        "generated plugin at {:?}",
        hyprland_plugin::PLUGIN_OUTPUT_PATH
    );
    plugin::load(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH))
        .context("unable to load plugin")?;
    trace!("loaded plugin");
    Ok(())
}
