use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::TempDir;
use tracing::{Level, span};

struct Config {
    overview_key: Option<String>,
    overview_mod: Option<String>,
    switch_mod: Option<String>,
}

pub fn configure(dir: &TempDir) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "configure", path =? dir.path()).entered();
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
            "_HYPRSHELL_PRINT_START_",
            if cfg!(debug_assertions) { "1" } else { "0" },
        ),
        ("_HYPRSHELL_OVERVIEW_MOD_KEYCODE_", "125"),
        ("_HYPRSHELL_OVERVIEW_KEY_KEYCODE_", "15"),
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
    Ok(())
}
