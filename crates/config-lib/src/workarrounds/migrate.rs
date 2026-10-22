use crate::CURRENT_WORKARROUND_VERSION;
use crate::workarrounds::io::save::write_io_workarrounds;
use crate::workarrounds::io::{ClassToIcon, Workarrounds, WorkarroundsFile};
use anyhow::{Context, bail};
use std::io::Read;
use std::path::Path;
use toml_edit::DocumentMut;
use tracing::{debug, info, warn};

pub fn migrate(config_file: &Path) -> anyhow::Result<WorkarroundsFile> {
    let old_version = get_config_version(config_file)?;
    let mut new_config = match old_version {
        0 => {
            info!(
                "Migrating workarrounds from version {old_version} to new version {CURRENT_WORKARROUND_VERSION}"
            );
            WorkarroundsFile::new_without_file(Workarrounds {
                version: crate::CURRENT_WORKARROUND_VERSION,
                class_to_icon: vec![ClassToIcon {
                    enabled: true,
                    class: "helium".into(),
                    icon: Path::new("helium-browser").into(),
                }],
            })
        }
        _ => bail!("Unsupported old config version {old_version}, cannot migrate"),
    };

    match write_io_workarrounds(config_file, &mut new_config, true) {
        Ok(()) => {
            debug!("New config written successfully");
        }
        Err(err) => {
            warn!("Failed to write new config!, please update it manually. \n{err:?}");
        }
    }
    Ok(new_config)
}

pub fn check_migration_needed(config_file: &Path) -> anyhow::Result<bool> {
    let version = get_config_version(config_file).context("Failed to get config version")?;
    if version > CURRENT_WORKARROUND_VERSION {
        bail!("Config version is newer than currently supported version");
    }
    Ok(version != CURRENT_WORKARROUND_VERSION)
}

fn get_config_version(config_file: &Path) -> anyhow::Result<u64> {
    if !config_file.exists() {
        debug!("Workarrounds file does not exist, using 0 -> 1 migration");
        return Ok(0);
    }

    let mut file = std::fs::File::open(config_file).with_context(|| {
        format!(
            "Failed to open TOML workarrounds at ({})",
            config_file.display()
        )
    })?;
    let mut content = String::new();
    file.read_to_string(&mut content).with_context(|| {
        format!(
            "Failed to read TOML workarrounds at ({})",
            config_file.display()
        )
    })?;
    let doc = content.parse::<DocumentMut>().with_context(|| {
        format!(
            "Failed to read TOML workarrounds at ({})",
            config_file.display()
        )
    })?;
    if let Some(version) = doc["version"].as_integer() {
        Ok(version as u64)
    } else {
        bail!(
            "Workarrounds file does not have a version specified! please remove this file and let a new one be generated"
        );
    }
}
