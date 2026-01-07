use crate::{PLUGIN_AUTHOR, PLUGIN_DESC, PLUGIN_NAME, PLUGIN_VERSION};
use anyhow::Context;
use core_lib::binds::generate_transfer;
use core_lib::transfer::TransferType;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use core_lib::util::get_daemon_socket_path_buff;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use tempfile::TempDir;
use tracing::debug_span;

pub struct SwitchBindConfig {
    pub key: Box<str>,
    pub mod_mask: u32,
    pub hold_mask: u8,
    pub command: Box<str>,
}

pub struct PluginConfig {
    pub switch_binds: Vec<SwitchBindConfig>,
    pub xkb_key_overview_mod: Option<Box<str>>,
    pub xkb_key_overview_key: Option<Box<str>>,
}
impl Display for PluginConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
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
            "$HYPRSHELL_OVERVIEW_MOD$",
            config.xkb_key_overview_mod.as_deref().unwrap_or(""),
        ),
        (
            "$HYPRSHELL_OVERVIEW_KEY$",
            config.xkb_key_overview_key.as_deref().unwrap_or(""),
        ),
        ("$HYPRSHELL_SWITCH_BINDS$", &switch_binds_defs(&config.switch_binds)),
        (
            "$HYPRSHELL_OPEN_OVERVIEW$",
            &generate_transfer(&TransferType::OpenOverview),
        ),
        (
            "$HYPRSHELL_CLOSE$",
            &generate_transfer(&TransferType::CloseSwitch),
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

impl PluginConfig {
    fn signature(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.switch_binds.len().hash(&mut hasher);
        for bind in &self.switch_binds {
            bind.key.hash(&mut hasher);
            bind.mod_mask.hash(&mut hasher);
            bind.hold_mask.hash(&mut hasher);
            bind.command.hash(&mut hasher);
        }
        self.xkb_key_overview_mod.hash(&mut hasher);
        self.xkb_key_overview_key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

fn switch_binds_defs(binds: &[SwitchBindConfig]) -> String {
    let count = binds.len();
    if count == 0 {
        return "\
#define HYPRSHELL_SWITCH_BIND_COUNT 0
static const char* HYPRSHELL_SWITCH_BIND_KEYS[1] = { \"\" };
static const uint32_t HYPRSHELL_SWITCH_BIND_MOD_MASKS[1] = { 0 };
static const uint8_t HYPRSHELL_SWITCH_BIND_HOLD_MASKS[1] = { 0 };
static const char* HYPRSHELL_SWITCH_BIND_COMMANDS[1] = { \"\" };
".to_string();
    }
    let keys = binds
        .iter()
        .map(|b| format!("\"{}\"", escape_c_string(&b.key)))
        .collect::<Vec<_>>()
        .join(", ");
    let mod_masks = binds
        .iter()
        .map(|b| b.mod_mask.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let hold_masks = binds
        .iter()
        .map(|b| b.hold_mask.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let commands = binds
        .iter()
        .map(|b| format!("R\"HYPR({})HYPR\"", b.command))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\
#define HYPRSHELL_SWITCH_BIND_COUNT {count}
static const char* HYPRSHELL_SWITCH_BIND_KEYS[HYPRSHELL_SWITCH_BIND_COUNT] = {{ {keys} }};
static const uint32_t HYPRSHELL_SWITCH_BIND_MOD_MASKS[HYPRSHELL_SWITCH_BIND_COUNT] = {{ {mod_masks} }};
static const uint8_t HYPRSHELL_SWITCH_BIND_HOLD_MASKS[HYPRSHELL_SWITCH_BIND_COUNT] = {{ {hold_masks} }};
static const char* HYPRSHELL_SWITCH_BIND_COMMANDS[HYPRSHELL_SWITCH_BIND_COUNT] = {{ {commands} }};
"
    )
}

fn escape_c_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
