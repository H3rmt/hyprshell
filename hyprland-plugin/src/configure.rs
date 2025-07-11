use anyhow::Context;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{Level, span};

pub fn configure<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "configure", path =? path.as_ref()).entered();
    let defs = path.as_ref().join("defs.hpp");

    let mut defs_file = OpenOptions::new()
        .read(true)
        .open(&defs)
        .with_context(|| format!("unable to open defs file: {defs:?}"))?;
    let mut buffer = String::new();
    defs_file
        .read_to_string(&mut buffer)
        .context("unable to read defs file")?;
    for replace in [
        ("@HYPRSHELL_PLUGIN_NAME@", env!("CARGO_PKG_NAME")),
        ("@HYPRSHELL_PLUGIN_AUTHOR@", env!("CARGO_PKG_AUTHORS")),
        ("@HYPRSHELL_PLUGIN_DESC@", env!("CARGO_PKG_DESCRIPTION")),
        ("@HYPRSHELL_PLUGIN_VERSION@", env!("CARGO_PKG_VERSION")),
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
