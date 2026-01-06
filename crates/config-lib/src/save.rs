use crate::Config;
use anyhow::{Context, bail};
use ron::Options;
use ron::extensions::Extensions;
use ron::ser::PrettyConfig;
use std::ffi::OsStr;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;
use tracing::{debug, debug_span, info};

const CONFIG_EXPLANATION: &str = "Edit with `hyprshell config edit`";

pub fn write_config(
    config_file: &Path,
    config: &Config,
    override_file: bool,
) -> anyhow::Result<()> {
    let _span = debug_span!("write_config").entered();
    let config_file_display = config_file.display();
    if config_file.exists() && !override_file {
        bail!(
            "Config file at {config_file_display} already exists, delete it before generating a new one or use -f to override"
        );
    }
    if let Some(parent) = config_file.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({})", parent.display()))?;
    }
    let str = match config_file.extension().and_then(OsStr::to_str) {
        None | Some("ron") => Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .with_default_extension(Extensions::UNWRAP_NEWTYPES)
            .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
            .to_string_pretty(config, PrettyConfig::default())
            .with_context(|| format!("Failed to write RON config to ({config_file_display})")),
        Some("json5" | "json") => {
            serde_json::to_string_pretty(config).context("Failed to generate JSON config")
        }
        Some("toml") => toml::to_string_pretty(config).context("Failed to generate TOML config"),
        Some(ext) => bail!(
            "Invalid config file extension: {ext} (run with -vv and check `FEATURES: ` debug log to see enabled extensions)"
        ),
    }?;
    #[allow(clippy::match_same_arms)]
    let header = match config_file.extension().and_then(OsStr::to_str) {
        None | Some("ron") => format!("// {CONFIG_EXPLANATION}"),
        Some("json5") => format!("// {CONFIG_EXPLANATION}"),
        Some("toml") => format!("# {CONFIG_EXPLANATION}"),
        _ => String::new(),
    };
    let content = format!("{header}\n{str}");
    let mut file = File::create(config_file)
        .with_context(|| format!("Failed to create config file at ({config_file_display})"))?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to config file at ({config_file_display})"))
        .inspect_err(|_| {
            info!("New config contents: {config:?}");
        })?;

    debug!("Config file written successfully at {config_file_display}");
    Ok(())
}
