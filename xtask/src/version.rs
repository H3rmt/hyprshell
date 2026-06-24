use anyhow::Context;
use std::fs;
use std::path::Path;
use tracing::{debug, info, trace};

use crate::WS;
use crate::cmd::run_cargo_command;
use crate::edit::edit_toml;
use crate::load::{load_toml, write_toml};

pub fn increment_versions(
    version: &semver::Version,
    member: &WS,
    dry_run: bool,
) -> anyhow::Result<()> {
    let path = member.path.join("Cargo.toml");
    info!("bumping version in {}", path.display());
    edit_toml(&path, "package.version", &version.to_string(), dry_run)
        .context(format!("Failed to edit {}", path.display()))?;
    Ok(())
}

pub fn increment_dependencies(
    version: &semver::Version,
    member: &WS,
    workspace_members: &[WS],
    dry_run: bool,
) -> anyhow::Result<()> {
    let path = Path::new(&member.path);
    let mut main_cargo = load_toml(&path.join("Cargo.toml"))
        .with_context(|| format!("Failed to load {}", path.display()))?;
    let deps = main_cargo
        .get_mut("dependencies")
        .and_then(|d| d.as_table_like_mut())
        .context("couldnt find dependencies")?;
    trace!("loaded {} direct dependencies", deps.len());
    update_dep_table(version, deps, path, workspace_members);
    let deps = main_cargo
        .get_mut("workspace")
        .and_then(|w| w.get_mut("dependencies"))
        .and_then(|d| d.as_table_like_mut())
        .context("couldnt find dependencies")?;
    trace!("loaded {} workspace dependencies", deps.len());
    update_dep_table(version, deps, path, workspace_members);
    if dry_run {
        info!(
            "Dry run: would update dependencies in {} Cargo.toml to version {version}. look at trace logs for details",
            member.name
        );
        trace!(
            "would write into {}: {}",
            path.join("Cargo.toml").display(),
            main_cargo.to_string()
        );
    } else {
        write_toml(&path.join("Cargo.toml"), &main_cargo)
            .context("Failed to write main Cargo.toml")?;
    }
    Ok(())
}

fn update_dep_table(
    version: &semver::Version,
    deps: &mut dyn toml_edit::TableLike,
    path: &Path,
    workspace_members: &[WS],
) {
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
                if workspace_members.iter().any(|ws| *ws.path == ppath) {
                    info!(
                        "bumping dependency {name} in {} to version {version}",
                        ppath.display()
                    );
                    dep_table.insert("version", format!("={version}").into());
                } else {
                    trace!(
                        "skipping dependency {name} because its path {p} does not match any workspace member in {workspace_members:?}"
                    );
                }
            } else {
                trace!("skipping dependency {name} because it is not a path dependency");
            }
        } else {
            trace!("skipping dependency {name} because it is not a table");
        }
    }
}

pub fn update_lockfile(dry_run: bool) -> anyhow::Result<()> {
    info!("updating Cargo.lock file");
    let args = &["update", "--workspace"];
    run_cargo_command(args, dry_run).context("Failed to update Cargo.lock file")?;
    Ok(())
}
