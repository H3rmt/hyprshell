use crate::load::load_config_file;
use crate::migrate::check::get_config_version;
use crate::{CURRENT_CONFIG_VERSION, migrate, write_config};
use anyhow::{Context, bail};
use std::path::Path;
use tracing::{debug_span, info, warn};

pub fn migrate(config_file: &Path) -> anyhow::Result<crate::Config> {
    let _span = debug_span!("migrate").entered();
    let old_version = get_config_version(config_file)?;

    let new_config = match old_version {
        migrate::m1t2::PREV_CONFIG_VERSION => {
            info!("Migrating from version {old_version} to new version {CURRENT_CONFIG_VERSION}");
            let old_config: migrate::m1t2::Config =
                load_config_file(config_file).context("Failed to load old config")?;
            let i1 = migrate::m2t3::Config::from(old_config);
            crate::Config::from(i1)
        }
        migrate::m2t3::PREV_CONFIG_VERSION => {
            info!("Migrating from version {old_version} to new version {CURRENT_CONFIG_VERSION}");
            let old_config: migrate::m2t3::Config =
                load_config_file(config_file).context("Failed to load old config")?;
            crate::Config::from(old_config)
        }
        _ => bail!("Unsupported old config version {old_version}, cannot migrate"),
    };
    match write_config(config_file, &new_config, true) {
        Ok(()) => {
            info!("New config written successfully");
        }
        Err(err) => {
            warn!("Failed to write new config!, please update it manually. \n{err:?}");
        }
    }
    Ok(new_config)
}
