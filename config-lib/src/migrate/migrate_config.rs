use crate::load::load_config_file;
use crate::migrate::check::get_config_version;
use crate::{CURRENT_CONFIG_VERSION, Config, migrate, write_config};
use anyhow::{Context, bail};
use std::path::Path;
use tracing::{Level, info, span, warn};

pub fn migrate(config_path: &Path) -> anyhow::Result<Config> {
    let _span = span!(Level::TRACE, "migrate").entered();
    let old_version = get_config_version(config_path)?;

    let new_config = match old_version {
        migrate::m1t2::PREV_CONFIG_VERSION => {
            info!("Migrating from version {old_version} to new version {CURRENT_CONFIG_VERSION}");
            let old_config: migrate::m1t2::Config =
                load_config_file(config_path).context("Failed to load old config")?;
            Config::from(old_config)
        }
        _ => bail!("Unsupported old config version {old_version}, cannot migrate"),
    };
    match write_config(config_path, &new_config, true) {
        Ok(_) => {
            info!("New config written successfully");
        }
        Err(err) => {
            warn!("Failed to write new config!, please update it manually {err:?}");
        }
    }
    Ok(new_config)
}
