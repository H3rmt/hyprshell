use anyhow::{Context, bail};
use std::io::Read;
use std::path::Path;
use tracing::instrument;

use crate::io::ConfigFile;

#[instrument(level = "debug", skip(allow_migrate))]
pub fn load_and_migrate_io_config(
    config_file: &Path,
    allow_migrate: bool,
) -> anyhow::Result<crate::io::ConfigFile> {
    if !config_file.exists() {
        bail!("Config file does not exist, create it using `hyprshell config generate`");
    }

    #[cfg(feature = "disable_migrations")]
    tracing::debug!("migrations disabled, not checking if updates are needed");

    #[cfg(not(feature = "disable_migrations"))]
    {
        use crate::migrate;

        if migrate::check_migration_needed(config_file)
            .inspect_err(|e| tracing::warn!("Failed to check if migration is needed: {e:?}"))
            .unwrap_or(false)
        {
            tracing::info!("Config needs migration");
            if !allow_migrate {
                bail!("Config file needs migration, but migration is not allowed.");
            }
            let migrated = crate::migrate::migrate(config_file);
            match migrated {
                Ok(config) => {
                    tracing::info!("Config migrated successfully");
                    let config = ConfigFile::try_from(config)
                        .context("Failed to convert TOML config to Config struct")?;
                    return Ok(config);
                }
                Err(err) => {
                    bail!("Config migration failed: \n{err:?}");
                }
            }
        }
        tracing::trace!("No migration needed");
    }
    #[cfg(feature = "disable_migrations")]
    {
        _ = allow_migrate;
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
    let document = (&content)
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to parse TOML config")?;
    let config =
        ConfigFile::try_from(document).context("Failed to convert TOML config to Config struct")?;

    tracing::debug!("Loaded config");
    Ok(config)
}

pub fn load_and_migrate_config(
    config_file: &Path,
    allow_migrate: bool,
) -> anyhow::Result<crate::Config> {
    let config = load_and_migrate_io_config(config_file, allow_migrate)?;

    let config: crate::Config = config
        .try_into()
        .context("Failed to convert config to internal format")?;

    // TODO move the check into the try from io config to exposed config
    crate::check(&config)?;

    Ok(config)
}
