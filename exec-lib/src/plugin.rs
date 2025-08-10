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

    // TODO get plugin list and check if the plugin with same config is already loaded
    unload();

    hyprland_plugin::generate(&PluginConfig {
        xkb_key_switch_mod: switch.map(|s| Box::from(mod_to_xkb_key(s))),
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

pub fn unload() {
    // let plugins = plugin::list().unwrap_or_default();
    // trace!("plugins: {:?}", plugins);
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

    let mut cmd = std::process::Command::new("sh");
    cmd.args([
        "-c",
        &format!(
            "hyprctl plugin unload {}",
            hyprland_plugin::PLUGIN_OUTPUT_PATH
        ),
    ]);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    debug!("Unloading plugin with command: {:?}", cmd);
    if let Err(e) = cmd.status() {
        debug!("Failed to unload plugin: {}", e);
    } else {
        debug!("Plugin unloaded successfully");
    }
}

pub const fn mod_to_xkb_key(r#mod: Modifier) -> &'static str {
    match r#mod {
        Modifier::Alt => "XKB_KEY_Alt",
        Modifier::Ctrl => "XKB_KEY_Control",
        Modifier::Super => "XKB_KEY_Super",
        Modifier::Shift => "XKB_KEY_Shift",
    }
}
