use crate::{CURRENT_CONFIG_VERSION, migrate::migrate_config::load_legacy_config_file};
use anyhow::{Context, bail};
use serde::Deserialize;
use std::path::Path;
use tracing::instrument;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EmptyConfig {
    pub(super) version: Option<u64>,
}

#[instrument(level = "debug")]
pub fn check_migration_needed(config_file: &Path) -> anyhow::Result<bool> {
    let version = get_config_version(config_file).context("Failed to get config version")?;
    if version > CURRENT_CONFIG_VERSION {
        bail!("Config version is newer than currently supported version");
    }
    Ok(version != CURRENT_CONFIG_VERSION)
}

#[instrument(level = "debug")]
pub fn get_config_version(config_file: &Path) -> anyhow::Result<u64> {
    if !config_file.exists() {
        bail!("Config file does not exist no need to migrate");
    }

    let config: EmptyConfig = load_legacy_config_file(config_file).with_context(|| {
        format!(
            "Failed to load config from file ({})",
            config_file.display()
        )
    })?;
    if let Some(version) = config.version {
        Ok(version)
    } else {
        bail!(
            "Config file does not have a version specified! please generate a new one using `hyprshell config generate`"
        );
    }
}
