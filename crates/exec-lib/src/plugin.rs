use anyhow::{Context, bail};
use config_lib::Modifier;
use hyprland::ctl::plugin;
use hyprland_plugin::PluginConfig;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{debug, debug_span, info, trace};

// info: trying to load a plugin causes hyprland to issue a reload
// this will cause hyprshell to restart.
// this second restart wont reload the plugin as the plugin config didnt change
// if the plugin fails to load it however tries again which the triggers the next reload
static PLUGIN_COULD_BE_BUILD: OnceLock<bool> = OnceLock::new();

pub fn load_plugin(
    switch: Option<(Modifier, Box<str>)>,
    overview: Option<(Modifier, Box<str>)>,
) -> anyhow::Result<()> {
    let _span = debug_span!("load_plugin").entered();

    if PLUGIN_COULD_BE_BUILD.get() == Some(&false) {
        bail!("plugin could not be built last, skipping to prevent reload loop");
    }

    let config = PluginConfig {
        xkb_key_switch_mod: switch
            .as_ref()
            .map(|(r#mod, _)| Box::from(mod_to_xkb_key(*r#mod))),
        xkb_key_switch_key: switch.map(|(_, key)| key),
        xkb_key_overview_mod: overview
            .as_ref()
            .map(|(r#mod, _)| Box::from(r#mod.to_string())),
        xkb_key_overview_key: overview.map(|(_, key)| key),
    };

    if check_new_plugin_needed(&config) {
        unload().context("unable to unload old plugin")?;
        info!("Building plugin, this may take a while, please wait");
        hyprland_plugin::generate(&config).context("unable to generate plugin")?;
        trace!(
            "generated plugin at {:?}",
            hyprland_plugin::PLUGIN_OUTPUT_PATH
        );
        if let Err(err) = plugin::load(Path::new(hyprland_plugin::PLUGIN_OUTPUT_PATH)) {
            PLUGIN_COULD_BE_BUILD.get_or_init(|| false);
            trace!("plugin failed to load, disabling plugin");
            bail!("unable to load plugin: {err:?}")
        }
        trace!("loaded plugin");
    } else {
        debug!("plugin already loaded, skipping");
    }

    Ok(())
}

pub fn check_new_plugin_needed(config: &PluginConfig) -> bool {
    let plugins = plugin::list().unwrap_or_default();
    trace!("plugins: {plugins:?}");
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

#[allow(clippy::must_use_candidate)]
pub const fn mod_to_xkb_key(r#mod: Modifier) -> &'static str {
    match r#mod {
        Modifier::Alt => "XKB_KEY_Alt",
        Modifier::Ctrl => "XKB_KEY_Control",
        Modifier::Super => "XKB_KEY_Super",
        Modifier::None => "XKB_KEY_NoSymbol",
    }
}
