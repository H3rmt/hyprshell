use anyhow::Context;
use config_lib::Modifier;
use hyprland::ctl::plugin;
use hyprland_plugin::PluginConfig;
use std::path::Path;
use tracing::{Level, debug, debug_span, span, trace};

pub fn load_plugin(modifier: Modifier) -> anyhow::Result<()> {
    let _span = debug_span!("load_plugin").entered();

    let plugins = plugin::list().unwrap_or_default();
    trace!("plugins: {:?}", plugins);
    // TODO
    // for plugin in plugins {
    //     if plugin.name == hyprland_plugin::PLUGIN_NAME {
    //         debug!("plugin already loaded, unloading it");
    //         plugin::unload(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH)).with_context(|| {
    //             format!(
    //                 "unable to unload old plugin at: {}",
    //                 hyprland_plugin::PLUGIN_OUTPUT_PATH
    //             )
    //         })?;
    //         debug!("plugin unloaded");
    //     }
    // }

    hyprland_plugin::generate(&PluginConfig {
        xkb_key_switch_mod: Box::from(mod_to_xkb_key(modifier)),
    })
    .context("unable to generate plugin")?;
    trace!(
        "generated plugin at {:?}",
        hyprland_plugin::PLUGIN_OUTPUT_PATH
    );
    plugin::load(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH))
        .context("unable to load plugin")?;
    trace!("loaded plugin");
    Ok(())
}

pub fn mod_to_xkb_key(r#mod: Modifier) -> &'static str {
    match r#mod {
        Modifier::Alt => "XKB_KEY_Alt",
        Modifier::Ctrl => "XKB_KEY_Control",
        Modifier::Super => "XKB_KEY_Super",
        Modifier::Shift => "XKB_KEY_Shift",
    }
}
