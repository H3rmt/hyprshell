use crate::config::Config;
use anyhow::{Context, bail};
use ron::Options;
use ron::extensions::Extensions;
use ron::ser::PrettyConfig;
use std::ffi::OsStr;
use std::fs::{File, create_dir_all};
use std::path::Path;
use tracing::{Level, info, span};

pub fn write_config(
    config_path: &Path,
    config: &Config,
    override_file: bool,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "write_config").entered();

    if config_path.exists() && !override_file {
        bail!(
            "Config file at {config_path:?} already exists, delete it before generating a new one or use -f to override"
        );
    }
    if let Some(parent) = config_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({parent:?})"))?;
    }
    match config_path.extension().and_then(OsStr::to_str) {
        None | Some("ron") => {
            let file = File::create(config_path)
                .with_context(|| format!("Failed to create config at ({config_path:?})"))?;
            Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .with_default_extension(Extensions::UNWRAP_NEWTYPES)
                .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
                .to_io_writer_pretty(file, config, PrettyConfig::default())
                .context("Failed to write ron config")?;
        }
        Some("json") => {
            let file = File::create(config_path)
                .with_context(|| format!("Failed to create config at ({config_path:?})"))?;
            serde_json::to_writer_pretty(file, config).context("Failed to write json config")?
        }
        #[cfg(feature = "toml_config")]
        Some("toml") => {
            use std::fs::write;
            let str = toml::to_string_pretty(config).context("Failed to write toml config")?;
            write(config_path, str)
                .with_context(|| format!("Failed to create config at ({config_path:?})"))?;
        }
        Some(ext) => bail!(
            "Invalid config file extension: {} (check `FEATURES: ` debug log to see enabled extensions)",
            ext
        ),
    };

    info!("Config file written successfully at {:?}", config_path);
    Ok(())
}
