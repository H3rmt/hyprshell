use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use core_lib::binds::generate_transfer_socat;
use core_lib::transfer::TransferType;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use tempfile::TempDir;
use tracing::debug_span;

pub struct PluginConfig {
    pub xkb_key_switch_mod: Box<str>,
}
impl Display for PluginConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.xkb_key_switch_mod)
    }
}

pub fn configure(dir: &TempDir, config: &PluginConfig) -> anyhow::Result<()> {
    let _span = debug_span!("configure", path =? dir.path()).entered();
    let defs = dir.path().join("defs.hpp");

    let mut defs_file = OpenOptions::new()
        .read(true)
        .open(&defs)
        .with_context(|| format!("unable to open defs file: {}", defs.display()))?;
    let mut buffer = String::new();
    defs_file
        .read_to_string(&mut buffer)
        .context("unable to read defs file")?;
    for replace in [
        ("#include \"defs-test.hpp\"", ""),
        ("$HYPRSHELL_PLUGIN_NAME$", PLUGIN_NAME),
        ("$HYPRSHELL_PLUGIN_AUTHOR$", PLUGIN_AUTHOR),
        (
            "$HYPRSHELL_PLUGIN_DESC$",
            &format!("{PLUGIN_DESC} - {config}"),
        ),
        ("$HYPRSHELL_PLUGIN_VERSION$", PLUGIN_VERSION),
        (
            "$HYPRSHELL_PRINT_DEBUG$",
            if cfg!(debug_assertions) { "1" } else { "0" },
        ),
        (
            "$HYPRSHELL_SWTICH_XKB_KEY_L$",
            &format!("{}_L", config.xkb_key_switch_mod),
        ),
        (
            "$HYPRSHELL_SWTICH_XKB_KEY_R$",
            &format!("{}_R", config.xkb_key_switch_mod),
        ),
        (
            "$HYPRSHELL_PROGRAM_CLOSE_SWITCH$",
            &generate_transfer_socat(&TransferType::CloseSwitch),
        ),
    ] {
        buffer = buffer.replace(replace.0, replace.1);
    }
    buffer.push('\n');
    drop(defs_file);
    let mut defs_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&defs)
        .with_context(|| format!("unable to open defs file: {}", defs.display()))?;
    defs_file
        .write_all(buffer.as_bytes())
        .context("unable to write defs file")?;
    // trace!("Updated defs file: {defs:?}, content:\n{buffer}");
    Ok(())
}
