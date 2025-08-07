use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use core_lib::binds::generate_transfer_socat;
use core_lib::transfer::TransferType;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use tempfile::TempDir;
use tracing::{Level, debug_span, span};

pub struct PluginConfig {
    pub switch_mod: String,
}

pub fn configure(dir: &TempDir, config: &PluginConfig) -> anyhow::Result<()> {
    let _span = debug_span!("configure", path =? dir.path()).entered();
    let defs = dir.path().join("defs.hpp");

    let mut defs_file = OpenOptions::new()
        .read(true)
        .open(&defs)
        .with_context(|| format!("unable to open defs file: {defs:?}"))?;
    let mut buffer = String::new();
    defs_file
        .read_to_string(&mut buffer)
        .context("unable to read defs file")?;
    for replace in [
        ("#include \"defs-test.hpp\"", ""),
        ("_HYPRSHELL_PLUGIN_NAME_", PLUGIN_NAME),
        ("_HYPRSHELL_PLUGIN_AUTHOR_", PLUGIN_AUTHOR),
        ("_HYPRSHELL_PLUGIN_DESC_", PLUGIN_DESC),
        ("_HYPRSHELL_PLUGIN_VERSION_", PLUGIN_VERSION),
        (
            "_HYPRSHELL_PRINT_DEBUG_",
            if cfg!(debug_assertions) { "1" } else { "0" },
        ),
        ("_HYPRSHELL_SWTICH_RELEASE_KEYCODE_", &config.switch_mod),
        (
            "_HYPRSHELL_PROGRAM_CLOSE_SWITCH_",
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
        .with_context(|| format!("unable to open defs file: {defs:?}"))?;
    defs_file
        .write_all(buffer.as_bytes())
        .context("unable to write defs file")?;
    // trace!("Updated defs file: {defs:?}, content:\n{buffer}");
    Ok(())
}
