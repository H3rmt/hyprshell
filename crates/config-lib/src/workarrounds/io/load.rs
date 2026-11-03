use crate::workarrounds::io::structs::WorkarroundsFile;
use std::{io::Read, path::Path};

use anyhow::{Context, bail};
use tracing::instrument;

#[instrument(level = "debug")]
pub fn load_and_migrate_workarounds(
    config_file: &Path,
    allow_migrate: bool,
) -> anyhow::Result<WorkarroundsFile> {
    #[cfg(feature = "disable_migrations")]
    tracing::debug!("migrations disabled, not checking if updates are needed");

    #[cfg(not(feature = "disable_migrations"))]
    {
        use crate::workarrounds::migrate;

        if migrate::check_migration_needed(config_file)
            .inspect_err(|e| tracing::warn!("Failed to check if migration is needed: {e:?}"))
            .unwrap_or(false)
        {
            tracing::info!("Workarrounds file needs migration");
            if !allow_migrate {
                bail!("Workarrounds file needs migration, but migration is not allowed.");
            }
            let migrated = migrate::migrate(config_file);
            match migrated {
                Ok(config) => {
                    tracing::info!("Workarrounds file migrated successfully");
                    return Ok(config);
                }
                Err(err) => {
                    bail!("Workarrounds file migration failed: \n{err:?}");
                }
            }
        }
        tracing::trace!("No migration needed");
    }
    #[cfg(feature = "disable_migrations")]
    {
        _ = allow_migrate;
        if !config_file.exists() {
            bail!("Workarrounds file does not exist, must be created using migrations");
        }
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
        .context("Failed to parse TOML workarrounds")?;
    let config = WorkarroundsFile::try_from(document)
        .context("Failed to convert TOML workarrounds to Workarrounds struct")?;

    tracing::debug!("Loaded workarrounds");
    Ok(config)
}
