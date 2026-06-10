use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing::{debug, info, trace};
use tracing_subscriber::EnvFilter;

use crate::edit::edit_toml;
use crate::load::{load_toml, write_toml};

pub fn increment_versions(version: &semver::Version, workspace: &[PathBuf]) -> anyhow::Result<()> {
    info!("bumping version in main Cargo.toml");
    edit_toml(
        Path::new("Cargo.toml"),
        "package.version",
        &version.to_string(),
    )
    .context("Failed to edit main Cargo.toml")?;
    for member in workspace {
        let path = member.join("Cargo.toml");
        info!("bumping version in {}", path.display());
        edit_toml(&path, "package.version", &version.to_string())
            .context(format!("Failed to edit {}", path.display()))?;
    }
    Ok(())
}

pub fn increment_dependencies(
    version: &semver::Version,
    workspace: &[PathBuf],
) -> anyhow::Result<()> {
    let path = Path::new("");
    let mut main_cargo =
        load_toml(&path.join("Cargo.toml")).context("Failed to load main Cargo.toml")?;
    // load workspace.dependencies
    // check each dependency, if it is a path dependency to a workspace member, update the version
    let deps = main_cargo
        .get_mut("dependencies")
        .and_then(|d| d.as_table_like_mut())
        .context("couldnt find dependencies")?;
    trace!("loaded {} direct dependencies", deps.len());
    check_dep_table(version, deps, &path, workspace)
        .context("Failed to check dependencies in main Cargo.toml")?;
    let deps = main_cargo
        .get_mut("workspace")
        .and_then(|w| w.get_mut("dependencies"))
        .and_then(|d| d.as_table_like_mut())
        .context("couldnt find dependencies")?;
    trace!("loaded {} workspace dependencies", deps.len());
    check_dep_table(version, deps, &path, workspace)
        .context("Failed to check dependencies in main Cargo.toml")?;
    write_toml(&path.join("Cargo.toml"), &main_cargo).context("Failed to write main Cargo.toml")?;

    for member in workspace {
        let path = Path::new(member);
        let mut main_cargo = load_toml(&path.join("Cargo.toml"))
            .with_context(|| format!("Failed to load {}", path.display()))?;
        let deps = main_cargo
            .get_mut("dependencies")
            .and_then(|d| d.as_table_like_mut())
            .context("couldnt find dependencies")?;
        trace!("loaded {} direct dependencies", deps.len());
        check_dep_table(version, deps, &path, workspace)
            .context("Failed to check dependencies in main Cargo.toml")?;
        write_toml(&path.join("Cargo.toml"), &main_cargo)
            .context("Failed to write main Cargo.toml")?;
    }
    Ok(())
}

fn check_dep_table(
    version: &semver::Version,
    deps: &mut dyn toml_edit::TableLike,
    path: &Path,
    workspace: &[PathBuf],
) -> anyhow::Result<()> {
    for (name, dep) in deps.iter_mut() {
        if let Some(dep_table) = dep.as_table_like_mut() {
            trace!("checking dependency: {name}");
            if let Some(p) = dep_table.get("path") {
                let p = p.as_str().unwrap_or_default();
                let Ok(ppath) = fs::canonicalize(Path::new(path).join(p)) else {
                    trace!(
                        "skipping dependency {name} because its path {p} does not resolve to a valid path"
                    );
                    continue;
                };
                debug!(
                    "found path dependency {name} with path {} in {}",
                    ppath.display(),
                    path.display()
                );
                // check if the path resolves to a workspace member, if so, update the version

                if workspace.iter().any(|ws| *ws == ppath) {
                    info!(
                        "bumping dependency {name} in {} to version {version}",
                        ppath.display()
                    );
                    dep_table.insert("version", format!("={}", version).into());
                } else {
                    trace!(
                        "skipping dependency {name} because its path {p} does not match any workspace member in {workspace:?}"
                    );
                }
            } else {
                trace!("skipping dependency {name} because it is not a path dependency");
            }
        } else {
            trace!("skipping dependency {name} because it is not a table");
        }
    }
    Ok(())
}

pub fn update_lockfile() -> anyhow::Result<()> {
    // run `cargo update` to update the Cargo.lock file
    info!("updating Cargo.lock file");
    let args = ["update", "--workspace"];
    debug!("running cargo with args: {args:?}");
    let status = std::process::Command::new("cargo")
        .args(args)
        .status()
        .context("Failed to run cargo update")?;
    if !status.success() {
        anyhow::bail!("cargo update failed with status: {status}");
    }
    Ok(())
}
