use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use crate::load::load_toml;

mod bundle;
mod cmd;
mod edit;
mod licenses;
mod load;
mod pkgbuild;
mod version;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Hyprshell task runner", long_about = None)]
pub struct AppArgs {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    /// Increase the verbosity level (-v: debug, -vv: trace)
    #[arg(short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Directory to run the command in (defaults to current directory)
    #[arg(long, global = true)]
    pub dir: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Release {
    /// Set the version of all workspace members to the specified version, and update path dependencies to match the new version.
    SetVersion {
        /// Dont make any changes, just print what would be done
        #[arg(long, global = true)]
        dry_run: bool,

        /// Version to set (e.g., "1.2.3")
        version: String,
    },
    /// Bundle files and folders into a tar.x archive using the `tar` command. The input files/folders can be bundled either with their original path or flat (i.e., all at the root of the archive).
    Bundle {
        /// Path to a file of folder to bundle flat (e.g., "dist/"). can be set multiple times to bundle multiple files/folders.
        #[arg(long, value_delimiter = ' ', num_args = 1..)]
        input_flat: Option<Vec<PathBuf>>,

        /// Path to a file of folder to bundle with original path (e.g., "dist/"). can be set multiple times to bundle multiple files/folders.
        #[arg(long, value_delimiter = ' ')]
        input_with_path: Option<Vec<PathBuf>>,

        /// Path where to place the bundled output
        output: String,
    },
    /// Publish the package to crates.io
    Publish {
        /// Dont make any changes, just print what would be done
        #[arg(long, global = true)]
        dry_run: bool,
    },
    /// Update and publish PKGBUILDS (version, sha256sums). This is useful for releasing new versions of the package to the AUR.
    Pkgbuild {
        /// Path to the PKGBUILD file to update
        pkgbuild: PathBuf,

        /// Dont make any changes, just print what would be done
        #[arg(long, global = true)]
        dry_run: bool,

        /// Commit message for the changes
        #[arg(long)]
        msg: String,

        /// AUR username and email to use for committing the changes
        #[arg(long)]
        username: Option<String>,

        /// AUR username and email to use for committing the changes
        #[arg(long)]
        email: Option<String>,
    },

    Licenses {
        /// List of allowed licenses [default: CC0-1.0, Apache-2.0, Apache-2.0 WITH LLVM-exception, MIT, ISC, BSD-3-Clause, Zlib, Unicode-3.0, MPL-2.0, LGPL-3.0-only, GPL-3.0-or-later]
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "CC0-1.0, Apache-2.0, Apache-2.0 WITH LLVM-exception, MIT, ISC, BSD-3-Clause, Zlib, Unicode-3.0, MPL-2.0, LGPL-3.0-only, GPL-3.0-or-later"
        )]
        licenses: Vec<String>,

        /// Output file to write the licenses to.
        #[arg(long, short, default_value = "THIRD_PARTY_NOTICES.md")]
        out: PathBuf,
    },
    /// Installs all needed dependencies for building the package.
    Dependencies,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Cmd {
    /// Run clippy on all workspace members with the specified profile, and optionally with --frozen.
    Check {
        #[clap(flatten)]
        pf: ProfileFrozen,
    },
    /// Run clippy with --fix on all workspace members with the specified profile, and optionally with --frozen. This will attempt to automatically fix any clippy warnings, but may not be able to fix all of them.
    Fix {
        #[clap(flatten)]
        pf: ProfileFrozen,
    },
    /// Run formatting checks on all workspace members with the specified profile, and optionally with --frozen.
    Lint {},
    /// Run tests on all workspace members with the specified profile, and optionally with --frozen. By default, tests are run with cargo nextest, but can be disabled with --no-nextest.
    Test {
        #[clap(flatten)]
        pf: ProfileFrozen,

        /// Whether to run tests with cargo nextest instead of the default cargo test. This requires that nextest is installed.
        #[arg(long)]
        no_nextest: bool,
    },
    /// Format all workspace members with the specified profile.
    Format {},
    // CheckNixDefaultConfig {},
    // CheckAllFeatureCombinations {},
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Commands used for / during releasing the package.
    Release {
        #[clap(subcommand)]
        command: Release,
    },

    /// Commands used for / during development, such as linting, formatting, and testing.
    Cmd {
        #[clap(subcommand)]
        command: Cmd,
    },
}

#[derive(Args, Debug, Clone)]
pub struct ProfileFrozen {
    /// Profile to use (e.g., "dev", "release"). Defaults to "dev".
    #[arg(short, long, default_value = "dev")]
    profile: String,
    /// Whether to run clippy with --locked, which requires that the Cargo.lock file is up-to-date.
    /// If the lock file is missing, or it needs to be updated, Cargo will exit with an error.
    #[arg(long)]
    locked: bool,
}

#[allow(clippy::too_many_lines)]
fn main() -> anyhow::Result<()> {
    let cli = AppArgs::parse();
    let opts = cli.global_opts;

    let level = match opts.verbose {
        0 => "info",
        1 => "debug",
        2.. => "trace",
    };
    tracing_log::LogTracer::init()
        .unwrap_or_else(|e| warn!("Unable to initialize log logging: {e}"));
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("hyprshell_xtask={level}").into());
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).context("Unable to set up logging")?;

    // Change to the specified directory if provided
    if let Some(dir) = opts.dir {
        debug!("changing directory to: {}", dir);
        env::set_current_dir(&dir).context("Failed to change directory")?;
    } else {
        debug!("running in directory: {}", env::current_dir()?.display());
    }

    match cli.command {
        Command::Release { command } => match command {
            Release::SetVersion { version, dry_run } => {
                let version =
                    semver::Version::parse(&version).context("Failed to parse version")?;
                let workspace = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: true,
                    include_xtask: false,
                })
                .context("Failed to get workspace members from main Cargo.toml")?;
                debug!("workspace members: {workspace:?}");
                for member in &workspace {
                    version::increment_versions(&version, member, dry_run)
                        .context("Failed to increment versions")?;
                    version::increment_dependencies(&version, member, &workspace, dry_run)
                        .context("Failed to increment dependencies")?;
                }
                debug!("updating pkgbuilds file");
                for path in [
                    "packaging/pkgbuild/PKGBUILD",
                    "packaging/pkgbuild/PKGBUILD-slim",
                    "packaging/pkgbuild/PKGBUILD-bin",
                ] {
                    pkgbuild::set_version_in_pkgbuild(Path::new(path), &version, dry_run)
                        .context("Failed to set version in PKGBUILD")?;
                }
                version::update_lockfile(dry_run).context("Failed to update Cargo.lock file")?;
            }
            Release::Bundle {
                output,
                input_with_path,
                input_flat,
            } => {
                let tmp_path = tmp_dir("bundle").context("Failed to create temporary directory")?;
                bundle::bundle(
                    &tmp_path,
                    &output,
                    input_with_path.as_ref(),
                    input_flat.as_ref(),
                )
                .context("Failed to bundle")?;

                let metadata = fs::metadata(&output).context("Failed to stat output")?;
                info!("created archive {} ({} bytes)", output, metadata.len());

                // remove the temporary directory
                fs::remove_dir_all(&tmp_path).context("Failed to remove temporary directory")?;
            }
            Release::Pkgbuild {
                pkgbuild,
                dry_run,
                email,
                username,
                msg,
            } => {
                let name = pkgbuild::get_pkgbuild_name(&pkgbuild)
                    .context("Failed to get package name from PKGBUILD")?;
                info!("Updating PKGBUILD for package {name}");
                let tmp_path =
                    tmp_dir("aur-update").context("Failed to create temporary directory")?;
                let private_key = env::var("AUR_PRIVATE_KEY")
                    .context("AUR_PRIVATE_KEY environment variable not set")?;
                let out = pkgbuild::clone_pkgbuild_to_tmp(&name, &tmp_path, &private_key)
                    .context("Failed to clone PKGBUILD to temporary directory")?;
                debug!(
                    "cloned PKGBUILD to temporary directory {}",
                    tmp_path.display()
                );
                fs::copy(&pkgbuild, out.join("PKGBUILD"))
                    .context("Failed to copy PKGBUILD to temporary directory")?;
                pkgbuild::update_sha256sums_in_pkgbuild(&out)
                    .context("Failed to update sha256sums in PKGBUILD")?;
                info!("Updated sha256sums in PKGBUILD");
                pkgbuild::update_srcinfo_in_pkgbuild(&out)
                    .context("Failed to update .SRCINFO in PKGBUILD")?;
                info!("Updated .SRCINFO in PKGBUILD");
                pkgbuild::commit_and_push_pkgbuild(
                    &out,
                    username.as_deref(),
                    email.as_deref(),
                    &msg,
                    dry_run,
                )
                .context("Failed to push PKGBUILD to AUR")?;
                info!("Comitted and pushed PKGBUILD to AUR");
                // remove the temporary directory
                if dry_run {
                    info!(
                        "Dry run: not removing temporary directory {}",
                        tmp_path.display()
                    );
                } else {
                    fs::remove_dir_all(&tmp_path)
                        .context("Failed to remove temporary directory")?;
                }
            }
            Release::Publish { dry_run } => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: true,
                    include_xtask: false,
                })
                .context("Failed to get workspace members from main Cargo.toml")?;
                let mut args = vec!["publish"];
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                args.extend(["--no-verify", "--locked"]);
                if dry_run {
                    args.push("--dry-run");
                }
                info!("Running cargo publish");
                let out =
                    cmd::run_cargo_command(&args, false).context("Failed to run cargo publish")?;
                if out != 0 {
                    anyhow::bail!("cargo publish failed with exit code {out}");
                }
            }
            Release::Licenses { licenses, out } => {
                let gen_ =
                    licenses::gen_licenses(&licenses).context("Failed to generate licenses")?;
                fs::write(out, &gen_).context("Failed to write licenses output")?;
                info!("Generated licenses output ({} lines)", gen_.lines().count());
            }
            Release::Dependencies => {}
        },
        Command::Cmd { command } => match command {
            Cmd::Check { pf } => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: false,
                    include_xtask: true,
                })
                .context("Failed to get workspace names")?;
                let mut args = vec![
                    "clippy",
                    "--profile",
                    &pf.profile,
                    "--all-targets",
                    "--no-deps",
                ];
                if pf.locked {
                    args.push("--locked");
                }
                args.extend(["-p", "hyprshell"]);
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                args.extend(["--", "--deny", "warnings"]);
                info!("Running clippy");
                let out = cmd::run_cargo_command(&args, false).context("Failed to run clippy")?;
                info!("clippy finished with exit code {out}");
                if out != 0 {
                    anyhow::bail!("clippy failed with exit code {out}");
                }
            }
            Cmd::Fix { pf } => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: false,
                    include_xtask: true,
                })
                .context("Failed to get workspace names")?;
                let mut args = vec![
                    "clippy",
                    "--profile",
                    &pf.profile,
                    "--all-targets",
                    "--no-deps",
                    "--fix",
                    "--allow-dirty",
                ];
                if pf.locked {
                    args.push("--locked");
                }
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                info!("Running clippy fix");
                let out =
                    cmd::run_cargo_command(&args, false).context("Failed to run clippy fix")?;
                info!("clippy fix finished with exit code {out}");
                if out != 0 {
                    anyhow::bail!("clippy fix failed with exit code {out}");
                }
            }
            Cmd::Lint {} => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: false,
                    include_xtask: true,
                })
                .context("Failed to get workspace names")?;
                let mut args = vec!["fmt"];
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                args.extend(["--", "--check"]);
                info!("Running fmt check");
                let out = cmd::run_cargo_command(&args, false).context("Failed to run fmt")?;
                info!("fmt check finished with exit code {out}");
                if out != 0 {
                    anyhow::bail!("fmt failed with exit code {out}");
                }
            }
            Cmd::Format {} => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: false,
                    include_xtask: true,
                })
                .context("Failed to get workspace names")?;
                let mut args = vec!["fmt"];
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                info!("Running fmt");
                let out = cmd::run_cargo_command(&args, false).context("Failed to run fmt")?;
                info!("fmt finished with exit code {out}");
                if out != 0 {
                    anyhow::bail!("fmt failed with exit code {out}");
                }
            }
            Cmd::Test { no_nextest, pf } => {
                let ws = get_workspace_names(&GetWorkspaceConfig {
                    include_base: true,
                    include_vendored_deps: false,
                    include_xtask: true,
                })
                .context("Failed to get workspace names")?;
                let mut args = if no_nextest {
                    vec!["test", "--profile", &pf.profile, "--all-targets"]
                } else {
                    vec![
                        "nextest",
                        "run",
                        "--cargo-profile",
                        &pf.profile,
                        "--all-targets",
                    ]
                };
                if pf.locked {
                    args.push("--locked");
                }
                args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
                info!("Running test/nextest");
                let out =
                    cmd::run_cargo_command(&args, false).context("Failed to run test/nextest")?;
                info!("test/nextest finished with exit code {out}");
                if out != 0 {
                    anyhow::bail!("test/nextest failed with exit code {out}");
                }
            }
        },
    }

    Ok(())
}

const DEP_CRATES_PREFIX: &str = "dep-crates/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WS {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GetWorkspaceConfig {
    include_vendored_deps: bool,
    include_xtask: bool,
    include_base: bool,
}

fn get_workspace_names(config: &GetWorkspaceConfig) -> anyhow::Result<Vec<WS>> {
    let main_cargo =
        load_toml(Path::new("Cargo.toml")).context("Failed to load main Cargo.toml")?;
    let workspace = main_cargo
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|m| {
            m.into_iter()
                .filter_map(|m| m.as_str())
                .filter(|m| config.include_xtask || !m.ends_with("xtask"))
                .filter(|m| config.include_vendored_deps || !m.starts_with(DEP_CRATES_PREFIX))
                .map(String::from)
                .filter_map(|m| {
                    let can = fs::canonicalize(Path::new(&m))
                        .inspect_err(|e| {
                            warn!("Failed to canonicalize path for workspace member {m}: {e:?}");
                        })
                        .ok()?;
                    Some(WS {
                        name: format!("hyprshell-{}", m.rsplit('/').next().unwrap_or(&m)),
                        path: can,
                    })
                })
                .collect::<Vec<_>>()
        })
        .context("Failed to get workspace members from main Cargo.toml")?;
    if config.include_base {
        let base = WS {
            name: "hyprshell".to_string(),
            path: ".".into(),
        };
        let mut workspaces = vec![base];
        workspaces.extend(workspace);
        return Ok(workspaces);
    }
    Ok(workspace)
}

fn tmp_dir(name: &str) -> anyhow::Result<PathBuf> {
    let dir = env::temp_dir();
    let tmp_path = dir.join(format!(
        "{}_{}-{}{}",
        env!("CARGO_PKG_NAME"),
        name,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            % 9999u64,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos()
            % 9999u128
    ));
    if tmp_path.exists() {
        warn!(
            "Temporary directory {} already exists, creating new one",
            tmp_path.display()
        );
        return tmp_dir(name);
    }
    fs::create_dir(&tmp_path).with_context(|| {
        format!(
            "Failed to create temporary directory at {}",
            tmp_path.display()
        )
    })?;
    Ok(tmp_path)
}
