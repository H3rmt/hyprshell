use crate::io::ConfigFile;
use crate::io::save::write_io_config;
use crate::migrate::check::get_config_version;
use crate::{CURRENT_CONFIG_VERSION, migrate};
use anyhow::{Context, bail};
use core_lib::WarnWithDetails;
use ron::Options;
use ron::extensions::Extensions;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::Path;
use tracing::{debug, info, warn};

pub fn migrate(config_file: &Path) -> anyhow::Result<ConfigFile> {
    let old_version = get_config_version(config_file)?;

    let mut new_config = match old_version {
        migrate::m1t2::PREV_CONFIG_VERSION => {
            info!(
                "Migrating config from version {old_version} to new version {CURRENT_CONFIG_VERSION}"
            );
            let old_config: migrate::m1t2::Config =
                load_legacy_config_file(config_file).context("Failed to load old config")?;
            let i1 = migrate::m2t3::Config::from(old_config);
            let i2 = migrate::m3t4::Config::from(i1);
            let i3 = migrate::m4t5::Config::from(i2);
            let i4 = crate::io::Config::from(i3);
            crate::io::ConfigFile::new_without_file(i4)
        }
        migrate::m2t3::PREV_CONFIG_VERSION => {
            info!(
                "Migrating config from version {old_version} to new version {CURRENT_CONFIG_VERSION}"
            );
            let old_config: migrate::m2t3::Config =
                load_legacy_config_file(config_file).context("Failed to load old config")?;
            let i1 = migrate::m3t4::Config::from(old_config);
            let i2 = migrate::m4t5::Config::from(i1);
            let i3 = crate::io::Config::from(i2);
            crate::io::ConfigFile::new_without_file(i3)
        }
        migrate::m3t4::PREV_CONFIG_VERSION => {
            info!(
                "Migrating config from version {old_version} to new version {CURRENT_CONFIG_VERSION}"
            );
            let old_config: migrate::m3t4::Config =
                load_legacy_config_file(config_file).context("Failed to load old config")?;
            let i1 = migrate::m4t5::Config::from(old_config);
            let i2 = crate::io::Config::from(i1);
            crate::io::ConfigFile::new_without_file(i2)
        }
        migrate::m4t5::PREV_CONFIG_VERSION => {
            info!(
                "Migrating config from version {old_version} to new version {CURRENT_CONFIG_VERSION}"
            );
            let old_config: migrate::m4t5::Config =
                load_legacy_config_file(config_file).context("Failed to load old config")?;
            let i1 = crate::io::Config::from(old_config);
            crate::io::ConfigFile::new_without_file(i1)
        }
        _ => bail!("Unsupported old config version {old_version}, cannot migrate"),
    };

    // config moved from one file to another
    let file_moved = !config_file.ends_with(".toml");

    // migrate all configs to toml
    let config_file_new = &config_file.with_extension("toml");

    let config_file_back = config_file.with_added_extension("bak");
    fs::copy(config_file, &config_file_back).with_context(|| {
        format!(
            "Failed to copy old config from {} to {}",
            config_file.display(),
            config_file_back.display()
        )
    })?;
    match write_io_config(config_file_new, &mut new_config, true) {
        Ok(()) => {
            debug!("New config written successfully");
            if file_moved {
                info!(
                    "Config was moved to {} from {}, old config is at {}",
                    config_file_new.display(),
                    config_file.display(),
                    config_file_back.display()
                );
                write_moved_config_message(config_file_new, config_file, &config_file_back)
                    .warn_details("failed to write moved message to old config file");
            }
        }
        Err(err) => {
            warn!("Failed to write new config!, please update it manually.\n{err:?}");
        }
    }
    Ok(new_config)
}

fn write_moved_config_message(
    config_file_new: &Path,
    config_file: &Path,
    config_file_back: &Path,
) -> anyhow::Result<()> {
    if !config_file.exists() {
        bail!("Old config file does not exist: {}", config_file.display());
    }

    let message = format!(
        "Config was moved to '{}'\nOld config is at '{}'\n",
        config_file_new.display(),
        config_file_back.display(),
    );
    fs::write(config_file, message).with_context(|| {
        format!(
            "Failed to write moved config message to {}",
            config_file.display()
        )
    })
}

pub(super) fn load_legacy_config_file<T: DeserializeOwned>(
    config_file: &Path,
) -> anyhow::Result<T> {
    let config_file_display = config_file.display();
    match config_file.extension().and_then(OsStr::to_str) {
        Some("ron") => {
            let options = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .with_default_extension(Extensions::UNWRAP_NEWTYPES)
                .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
            let file = std::fs::File::open(config_file)
                .with_context(|| format!("Failed to open RON config at ({config_file_display})"))?;
            options
                .from_reader(file)
                .with_context(|| format!("Failed to read RON config at ({config_file_display})"))
        }
        Some("json") => {
            let file = std::fs::File::open(config_file).with_context(|| {
                format!("Failed to open JSON5 config at ({config_file_display})")
            })?;
            serde_json::from_reader(file)
                .with_context(|| format!("Failed to read JSON5 config at ({config_file_display})"))
        }
        Some("toml") => {
            let mut file = std::fs::File::open(config_file).with_context(|| {
                format!("Failed to open TOML config at ({config_file_display})")
            })?;
            let mut content = String::new();
            file.read_to_string(&mut content).with_context(|| {
                format!("Failed to read TOML config at ({config_file_display})")
            })?;
            toml::from_str(&content).context("Failed to parse TOML config")
        }
        Some(ext) => bail!("Invalid config file extension: {ext}"),
        None => bail!("Invalid config file extension (no extension)"),
    }
}
