use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use core_lib::binds::generate_transfer;
use core_lib::transfer::{OpenSwitch, TransferType};
use core_lib::util::get_daemon_socket_path_buff;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use tempfile::TempDir;
use tracing::debug_span;

pub struct PluginConfig {
    pub xkb_key_switch_mod: Option<Box<str>>,
    pub xkb_key_switch_key: Option<Box<str>>,
    pub xkb_key_overview_mod: Option<Box<str>>,
    pub xkb_key_overview_key: Option<Box<str>>,
}
impl Display for PluginConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.xkb_key_switch_mod.as_deref().unwrap_or(""),
            self.xkb_key_switch_key.as_deref().unwrap_or(""),
            self.xkb_key_overview_mod.as_deref().unwrap_or(""),
            self.xkb_key_overview_key.as_deref().unwrap_or(""),
        )
    }
}

pub fn configure(dir: &TempDir, config: &PluginConfig) -> anyhow::Result<()> {
    let _span = debug_span!("configure", path =? dir.path()).entered();
    let defs = dir.path().join("defs.h");

    let mut defs_file = OpenOptions::new()
        .read(true)
        .open(&defs)
        .with_context(|| format!("unable to open defs file: {}", defs.display()))?;
    let mut buffer = String::new();
    defs_file
        .read_to_string(&mut buffer)
        .context("unable to read defs file")?;
    let path = get_daemon_socket_path_buff()
        .to_str()
        .map(str::to_string)
        .context("unable to get daemon socket path")?;
    for replace in [
        ("#include \"defs-test.h\"", ""),
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
        ("$HYPRSHELL_SOCKET_PATH$", &path),
        (
            "$HYPRSHELL_SWTICH_XKB_MOD_L$",
            &config
                .xkb_key_switch_mod
                .as_deref()
                .map_or_else(|| "-1".to_string(), |m| format!("{m}_L")),
        ),
        (
            "$HYPRSHELL_SWTICH_XKB_MOD_R$",
            &config
                .xkb_key_switch_mod
                .as_deref()
                .map_or_else(|| "-1".to_string(), |m| format!("{m}_R")),
        ),
        (
            "$HYPRSHELL_OVERVIEW_MOD$",
            config.xkb_key_overview_mod.as_deref().unwrap_or(""),
        ),
        (
            "$HYPRSHELL_OVERVIEW_KEY$",
            config.xkb_key_overview_key.as_deref().unwrap_or(""),
        ),
        (
            "$HYPRSHELL_SWITCH_KEY$",
            config.xkb_key_switch_key.as_deref().unwrap_or(""),
        ),
        (
            "$HYPRSHELL_OPEN_OVERVIEW$",
            &generate_transfer(&TransferType::OpenOverview),
        ),
        (
            "$HYPRSHELL_CLOSE$",
            &generate_transfer(&TransferType::CloseSwitch),
        ),
        (
            "$HYPRSHELL_OPEN_SWITCH$",
            &generate_transfer(&TransferType::OpenSwitch(OpenSwitch { reverse: false })),
        ),
        (
            "$HYPRSHELL_OPEN_SWITCH_REVERSE$",
            &generate_transfer(&TransferType::OpenSwitch(OpenSwitch { reverse: true })),
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
    // tracing::trace!("Updated defs file: {defs:?}, content:\n{buffer}");
    Ok(())
}
