use crate::CURRENT_CONFIG_VERSION;
use crate::load::load_config_file;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::path::Path;
use tracing::debug_span;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EmptyConfig {
    pub(super) version: Option<u16>,
}

pub fn check_migration_needed(config_path: &Path) -> anyhow::Result<bool> {
    let _span = debug_span!("check_migration_needed").entered();
    let version = get_config_version(config_path).context("Failed to get config version")?;
    Ok(version != CURRENT_CONFIG_VERSION)
}

pub fn get_config_version(config_path: &Path) -> anyhow::Result<u16> {
    let _span = debug_span!("get_config_version").entered();
    if !config_path.exists() {
        bail!("Config file does not exist no need to migrate");
    }

    let config: EmptyConfig = load_config_file(config_path).with_context(|| {
        format!(
            "Failed to load config from file ({})",
            config_path.display()
        )
    })?;
    if let Some(version) = config.version {
        Ok(version)
    } else {
        bail!("Config file does not have a version, unable to determine if migration need");
    }
}
