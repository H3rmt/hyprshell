use anyhow::{Context, bail};
use config_lib::{KeyCombo, KeyMod, Modifier, Switch};
use core_lib::binds::generate_transfer;
use core_lib::transfer::{HoldMod, OpenSwitch, TransferType};
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
    switches: &[Switch],
    overview: Option<(Modifier, Box<str>)>,
) -> anyhow::Result<()> {
    let _span = debug_span!("load_plugin").entered();

    if PLUGIN_COULD_BE_BUILD.get() == Some(&false) {
        bail!("plugin could not be built last, skipping to prevent reload loop");
    }

    let config = PluginConfig {
        switch_binds: collect_switch_binds(switches),
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

fn collect_switch_binds(switches: &[Switch]) -> Vec<hyprland_plugin::SwitchBindConfig> {
    let mut binds = Vec::new();
    for (profile, switch) in switches.iter().enumerate() {
        for combo in &switch.binds.forward {
            if let Some(bind) = build_bind(profile, combo, false) {
                binds.push(bind);
            }
        }
        for combo in &switch.binds.reverse {
            if let Some(bind) = build_bind(profile, combo, true) {
                binds.push(bind);
            }
        }
    }
    binds
}

fn build_bind(
    profile: usize,
    combo: &KeyCombo,
    reverse: bool,
) -> Option<hyprland_plugin::SwitchBindConfig> {
    let mod_mask = mod_mask(&combo.mods);
    let hold_mods = combo_hold_mods(combo);
    let hold_mask = hold_mask(&hold_mods);
    let command = generate_transfer(&TransferType::OpenSwitch(OpenSwitch {
        reverse,
        profile,
        hold_mods,
    }));
    Some(hyprland_plugin::SwitchBindConfig {
        key: combo.key.clone(),
        mod_mask,
        hold_mask,
        command: command.into_boxed_str(),
    })
}

fn combo_hold_mods(combo: &KeyCombo) -> Vec<HoldMod> {
    let mods = combo
        .hold_mods
        .as_deref()
        .unwrap_or(&combo.mods);
    mods.iter()
        .filter_map(|m| match m {
            KeyMod::Alt => Some(HoldMod::Alt),
            KeyMod::Ctrl => Some(HoldMod::Ctrl),
            KeyMod::Super => Some(HoldMod::Super),
            KeyMod::Shift => None,
        })
        .collect()
}

fn mod_mask(mods: &[KeyMod]) -> u32 {
    let mut mask = 0u32;
    for m in mods {
        mask |= match m {
            KeyMod::Alt => MOD_ALT,
            KeyMod::Ctrl => MOD_CTRL,
            KeyMod::Super => MOD_SUPER,
            KeyMod::Shift => MOD_SHIFT,
        };
    }
    mask
}

fn hold_mask(mods: &[HoldMod]) -> u8 {
    let mut mask = 0u8;
    for m in mods {
        mask |= match m {
            HoldMod::Alt => HOLD_ALT,
            HoldMod::Ctrl => HOLD_CTRL,
            HoldMod::Super => HOLD_SUPER,
        };
    }
    mask
}

const MOD_ALT: u32 = 1 << 0;
const MOD_CTRL: u32 = 1 << 1;
const MOD_SUPER: u32 = 1 << 2;
const MOD_SHIFT: u32 = 1 << 3;

const HOLD_ALT: u8 = 1 << 0;
const HOLD_CTRL: u8 = 1 << 1;
const HOLD_SUPER: u8 = 1 << 2;
