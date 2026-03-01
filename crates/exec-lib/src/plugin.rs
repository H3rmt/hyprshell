use anyhow::{Context, bail};
use config_lib::Modifier;
use core_lib::WarnWithDetails;
use hyprland::ctl::plugin;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::field::debug;
use tracing::{debug, debug_span, info, trace};

// info: trying to load a plugin causes hyprland to issue a reload
// this will cause hyprshell to restart.
// this second restart wont reload the plugin as the plugin config didnt change
// if the plugin fails to load it however tries again which the triggers the next reload
static PLUGIN_COULD_BE_BUILD: OnceLock<bool> = OnceLock::new();

static PLUGINS_DIR: &str = "hyprland-plugin/plugins/";
static PLUGIN_SRC_DIR: &str = "hyprland-plugin/plugin-files/";
// we always need a specific path
static STATIC_PLUGIN_PATH: &str = "hyprland-plugin/plugin.so";

pub fn load_plugin(cache_dir: &Path) -> anyhow::Result<()> {
    let _span = debug_span!("load_plugin").entered();

    // TODO check if needed any more
    if PLUGIN_COULD_BE_BUILD.get() == Some(&false) {
        bail!("plugin could not be built last, skipping to prevent reload loop");
    }

    // plugin was loaded and is up to date
    if unload_if_needed(cache_dir).context("unable to unload old plugin")? {
        return Ok(());
    }
    info!("Building plugin, this may take a while, please wait");

    let plugin_files_path = cache_dir.join(PLUGINS_DIR);
    fs::create_dir_all(&plugin_files_path).with_context(|| {
        format!(
            "unable to create plugin directory at: {}",
            plugin_files_path.display()
        )
    })?;

    let plugin_specific_path = plugin_files_path.join(format!(
        "plugin_{}_{}.so",
        &crate::util::get_version()
            .ok()
            .and_then(|d| d.version)
            .unwrap_or_else(|| "?.?.?".to_string())
            .replace('.', "-"),
        env!("CARGO_PKG_VERSION").replace('.', "-")
    ));

    let static_plugin_path = cache_dir.join(STATIC_PLUGIN_PATH);
    if plugin_specific_path.exists() && static_plugin_path.exists() && !cfg!(debug_assertions) {
        debug!("plugin already exists, skipping building");
    } else {
        let plugin_dir = cache_dir.join(PLUGIN_SRC_DIR);
        fs::remove_dir_all(&plugin_dir).warn_details("failed to remove old plugin dir, ignoring");

        trace!("extracting plugin from zip");
        hyprland_plugin::extract_plugin(&plugin_dir).context("Failed to extract plugin")?;
        trace!("building plugin");
        hyprland_plugin::build_plugin(&plugin_dir, &plugin_specific_path)
            .context("Failed to build plugin")?;
        trace!("generated plugin at {}", plugin_specific_path.display());

        fs::remove_file(&static_plugin_path).warn_details("failed to remove old plugin, ignoring");
        // Create relative path from symlink to target
        let relative_target = PathBuf::from("plugins").join(
            plugin_specific_path
                .file_name()
                .context("Failed to get plugin filename")?,
        );
        std::os::unix::fs::symlink(&relative_target, &static_plugin_path)
            .context("Failed to create symlink")?;
        trace!(
            "created symlink from {} to {}",
            static_plugin_path.display(),
            plugin_specific_path.display()
        );
    }

    if let Err(err) = plugin::load(&static_plugin_path) {
        PLUGIN_COULD_BE_BUILD.get_or_init(|| false);
        trace!("plugin failed to load, disabling plugin");
        bail!("unable to load plugin: {err:?}")
    } else {
        trace!("loaded plugin");
    }

    Ok(())
}

pub const OLD_PLUGIN_OUTPUT_PATH: &str = "/tmp/hyprshell.so";

pub fn unload_if_needed(cache_dir: &Path) -> anyhow::Result<bool> {
    let plugin_file = cache_dir.join(STATIC_PLUGIN_PATH);
    let plugins = plugin::list().unwrap_or_default();
    for plugin in plugins {
        if plugin.name == hyprland_plugin::PLUGIN_NAME {
            debug!("plugin found, checking if unload needed");
            if plugin.version == env!("CARGO_PKG_VERSION") {
                debug!("plugin is up to date, skipping unload");
                return Ok(true);
            }

            debug!("plugin loaded, unloading it");

            // unload the legacy plugin if it exists
            let _ = plugin::unload(Path::new(OLD_PLUGIN_OUTPUT_PATH));

            plugin::unload(&plugin_file).with_context(|| {
                format!("unable to unload old plugin at: {}", plugin_file.display())
            })?;
            debug!("plugin unloaded");
        }
    }
    Ok(false)
}
