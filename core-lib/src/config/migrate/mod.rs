use crate::config;
use crate::config::save::write_config;
use anyhow::{Context, bail};
use ron::Options;
use ron::extensions::Extensions;
use std::ffi::OsStr;
use std::path::Path;
use tracing::{Level, span};

mod convert;
mod old_structs;

pub fn migrate(config_path: &Path) -> anyhow::Result<config::Config> {
    let _span = span!(Level::TRACE, "migrate_if_needed").entered();
    match load_old_config(config_path) {
        Ok(old_config) => {
            let new_config = config::Config::from(old_config);
            write_config(config_path, &new_config, true)?;
            Ok(new_config)
        }
        Err(e) => {
            bail!("Failed to load old config for migration: {e:?}");
        }
    }
}

fn load_old_config(config_path: &Path) -> anyhow::Result<old_structs::Config> {
    let _span = span!(Level::TRACE, "load_old_config").entered();
    if !config_path.exists() {
        bail!("Config file does not exist no need to migrate");
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
                .context("Failed to read ron config")?
        }
        Some("json") => {
            let file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open config at ({config_path:?})"))?;
            serde_json::from_reader(file).context("Failed to read json config")?
        }
        #[cfg(feature = "toml_config")]
        Some("toml") => {
            use std::io::Read;
            let mut file = std::fs::File::open(config_path)
                .with_context(|| format!("Failed to open config at ({config_path:?})"))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .context("Failed to read toml config")?;
            toml::from_str(&content).context("Failed to parse toml config")?
        }
        Some(ext) => bail!(
            "Invalid config file extension: {} (check `FEATURES: ` debug log to see enabled extensions)",
            ext
        ),
    };

    Ok(config)
}
