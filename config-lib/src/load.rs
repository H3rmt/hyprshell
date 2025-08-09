use crate::Config;
use crate::migrate::check_migration_needed;
use anyhow::{Context, bail};
use ron::Options;
use ron::extensions::Extensions;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::path::Path;
use tracing::{debug, debug_span, info, trace, warn};

pub fn load_and_migrate_config(config_path: &Path, allow_migrate: bool) -> anyhow::Result<Config> {
    let _span = debug_span!("load_config", path =? config_path).entered();
    if !config_path.exists() {
        bail!("Config file does not exist, create it using `hyprshell config generate`");
    }

    if check_migration_needed(config_path)
        .inspect_err(|e| warn!("Failed to check if migration is needed: {e:?}"))
        .unwrap_or(false)
    {
        info!("Config needs migration");
        if !allow_migrate {
            bail!("Config file needs migration, but migration is not allowed.");
        }
        let migrated = crate::migrate::migrate(config_path);
        match migrated {
            Ok(config) => {
                info!("Config migrated successfully");
                crate::check(&config)?;
                return Ok(config);
            }
            Err(err) => {
                warn!("Migration failed: \n{err:?}");
            }
        }
    } else {
        trace!("No migration needed");
    }

    let config: Config = load_config_file(config_path).with_context(|| {
        format!(
            "Failed to load config from file ({})",
            config_path.display()
        )
    })?;
    debug!("Loaded config");

    crate::check(&config)?;

    Ok(config)
}

pub fn load_config_file<T: DeserializeOwned>(config_path: &Path) -> anyhow::Result<T> {
    let config_path_display = config_path.display();
    match config_path.extension().and_then(OsStr::to_str) {
        None | Some("ron") => {
            let options = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .with_default_extension(Extensions::UNWRAP_NEWTYPES)
                .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
            let file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open RON config at ({config_path_display})"))?;
            options
                .from_reader(file)
                .with_context(|| format!("Failed to read RON config at ({config_path_display})"))
        }
        #[cfg(not(feature = "json5_config"))]
        Some("json") => {
            let file = std::fs::File::open(config_path).with_context(|| {
                format!("Failed to open JSON5 config at ({config_path_display})")
            })?;
            serde_json::from_reader(file)
                .with_context(|| format!("Failed to read JSON5 config at ({config_path_display})"))
        }
        #[cfg(feature = "json5_config")]
        Some("json5" | "json") => {
            let file = std::fs::File::open(config_path).with_context(|| {
                format!("Failed to open JSON5 config at ({config_path_display})")
            })?;
            serde_json5::from_reader(file)
                .with_context(|| format!("Failed to read JSON5 config at ({config_path_display})"))
        }
        #[cfg(feature = "toml_config")]
        Some("toml") => {
            use std::io::Read;
            let mut file = std::fs::File::open(config_path).with_context(|| {
                format!("Failed to open TOML config at ({config_path_display})")
            })?;
            let mut content = String::new();
            file.read_to_string(&mut content).with_context(|| {
                format!("Failed to read TOML config at ({config_path_display})")
            })?;
            toml::from_str(&content).context("Failed to parse TOML config")
        }
        Some(ext) => bail!(
            "Invalid config file extension: {} (run with -vv and check `FEATURES: ` debug log to see enabled extensions)",
            ext
        ),
    }
}
