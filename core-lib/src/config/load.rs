use crate::config;
use crate::config::{Config, check};
use anyhow::{Context, bail};
use ron::Options;
use ron::extensions::Extensions;
use std::ffi::OsStr;
use std::path::Path;
use tracing::{Level, debug, info, span, warn};

pub fn load_config(config_path: &Path) -> anyhow::Result<Config> {
    let _span = span!(Level::TRACE, "load_config", path =? config_path).entered();
    if !config_path.exists() {
        bail!("Config file does not exist, create it using `hyprshell config generate`");
    }
    let config = match config_path.extension().and_then(OsStr::to_str) {
        None | Some("ron") => {
            let options = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .with_default_extension(Extensions::UNWRAP_NEWTYPES)
                .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
            let file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open config at ({config_path:?})"))?;
            options
                .from_reader(file)
                .context("Failed to read ron config")
        }
        Some("json") => {
            let file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open config at ({config_path:?})"))?;
            serde_json::from_reader(file).context("Failed to read json config")
        }
        #[cfg(feature = "toml_config")]
        Some("toml") => {
            use std::io::Read;
            let mut file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open config at ({config_path:?})"))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .context("Failed to read toml config")?;
            toml::from_str(&content).context("Failed to parse toml config")
        }
        Some(ext) => bail!(
            "Invalid config file extension: {} (check `FEATURES: ` debug log to see enabled extensions)",
            ext
        ),
    };

    let config = match config {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("Failed to load config: {err:?}");
            info!("Attempting to migrate config");
            let migrated = config::migrate::migrate(config_path);
            match migrated {
                Ok(cfg) => {
                    info!("Config migrated successfully");
                    cfg
                }
                Err(err) => {
                    warn!("Migration failed: {err:?}");
                    bail!("Failed to load config and migration failed");
                }
            }
        }
    };
    debug!("Loaded config");

    check(&config)?;

    Ok(config)
}
