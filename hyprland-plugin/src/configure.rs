use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::TempDir;
use tracing::{Level, span};

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
        ("@HYPRSHELL_PLUGIN_NAME@", PLUGIN_NAME),
        ("@HYPRSHELL_PLUGIN_AUTHOR@", PLUGIN_AUTHOR),
        ("@HYPRSHELL_PLUGIN_DESC@", PLUGIN_DESC),
        ("@HYPRSHELL_PLUGIN_VERSION@", PLUGIN_VERSION),
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
