use crate::CURRENT_CONFIG_VERSION;
use crate::load::load_config_file;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::path::Path;
use tracing::{Level, span};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EmptyConfig {
    pub(super) version: u16,
}

pub fn check_migration_needed(config_path: &Path) -> anyhow::Result<bool> {
    let _span = span!(Level::TRACE, "check_migration_needed").entered();
    let version = get_config_version(config_path).context("Failed to get config version")?;
    Ok(version != CURRENT_CONFIG_VERSION)
}

pub(crate) fn get_config_version(config_path: &Path) -> anyhow::Result<u16> {
    let _span = span!(Level::TRACE, "get_config_version").entered();
    if !config_path.exists() {
        bail!("Config file does not exist no need to migrate");
    }

    let config: EmptyConfig = load_config_file(config_path)
        .with_context(|| format!("Failed to load config from file ({config_path:?})"))?;
    Ok(config.version)
}
