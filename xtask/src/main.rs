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
mod load;
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
pub enum Command {
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
        /// Path where to place the bundled output
        #[arg(long)]
        output: String,

        /// Path to a file of folder to bundle flat (e.g., "dist/"). can be set multiple times to bundle multiple files/folders.
        #[arg(long, value_delimiter = ' ', num_args = 1..)]
        input_flat: Option<Vec<PathBuf>>,

        /// Path to a file of folder to bundle with original path (e.g., "dist/"). can be set multiple times to bundle multiple files/folders.
        #[arg(long, value_delimiter = ' ')]
        input_with_path: Option<Vec<PathBuf>>,
    },
    /// Run clippy on all workspace members with the specified profile, and optionally with --frozen.
    Check {
        #[clap(flatten)]
        pf: ProfileFrozen,
    },
    /// Run clippy with --fix on all workspace members with the specified profile, and optionally with --frozen. This will attempt to automatically fix any clippy warnings, but may not be able to fix all of them.
    ClippyFix {
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

#[derive(Args, Debug, Clone)]
pub struct ProfileFrozen {
    /// Profile to use for linting (e.g., "dev", "release", "check"). Defaults to "dev".
    #[arg(short, long, default_value = "dev")]
    profile: String,
    /// Whether to run clippy with --frozen, which prevents Cargo from accessing the network and requires a Cargo.lock file.
    #[arg(long)]
    frozen: bool,
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
        Command::SetVersion { version, dry_run } => {
            // Parse version into semver
            let version = semver::Version::parse(&version).context("Failed to parse version")?;
            let workspace_paths = get_workspace_names(true, false)
                .context("Failed to get workspace members from main Cargo.toml")?;
            debug!("workspace members: {workspace_paths:?}");
            version::increment_versions(&version, &workspace_paths, dry_run)
                .context("Failed to increment versions")?;
            version::increment_dependencies(&version, &workspace_paths, dry_run)
                .context("Failed to increment dependencies")?;
            version::update_lockfile(dry_run).context("Failed to update Cargo.lock file")?;
        }
        Command::Bundle {
            output,
            input_with_path,
            input_flat,
        } => {
            let dir = env::temp_dir();
            let tmp_path = dir.join(format!(
                "{}-{}-{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_nanos()
                    % 9999u128
            ));
            fs::create_dir(&tmp_path).with_context(|| {
                format!(
                    "Failed to create temporary directory at {}",
                    tmp_path.display()
                )
            })?;

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
        Command::Check { pf } => {
            let ws = get_workspace_names(false, true).context("Failed to get workspace names")?;
            let mut args = vec![
                "clippy",
                "--profile",
                &pf.profile,
                "--all-targets",
                "--no-deps",
            ];
            if pf.frozen {
                args.push("--frozen");
            }
            args.extend(["-p", "hyprshell"]);
            args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
            args.extend(["--", "--deny", "warnings"]);
            info!("Running clippy");
            let out = cmd::run_cargo_command(&args, false).context("Failed to run clippy")?;
            if out != 0 {
                anyhow::bail!("clippy failed with exit code {out}");
            }
        }
        Command::ClippyFix { pf } => {
            let ws = get_workspace_names(false, true).context("Failed to get workspace names")?;
            let mut args = vec![
                "clippy",
                "--profile",
                &pf.profile,
                "--all-targets",
                "--no-deps",
                "--fix",
                "--allow-dirty",
            ];
            if pf.frozen {
                args.push("--frozen");
            }
            args.extend(["-p", "hyprshell"]);
            args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
            info!("Running clippy fix");
            let out = cmd::run_cargo_command(&args, false).context("Failed to run clippy fix")?;
            if out != 0 {
                anyhow::bail!("clippy fix failed with exit code {out}");
            }
        }
        Command::Lint {} => {
            let ws = get_workspace_names(false, true).context("Failed to get workspace names")?;
            let mut args = vec!["fmt"];
            args.extend(["-p", "hyprshell"]);
            args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
            args.extend(["--", "--check"]);
            info!("Running fmt");
            let out = cmd::run_cargo_command(&args, false).context("Failed to run fmt")?;
            if out != 0 {
                anyhow::bail!("fmt failed with exit code {out}");
            }
        }
        Command::Format {} => {
            let ws = get_workspace_names(false, true).context("Failed to get workspace names")?;
            let mut args = vec!["fmt"];
            args.extend(["-p", "hyprshell"]);
            args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
            info!("Running fmt");
            let out = cmd::run_cargo_command(&args, false).context("Failed to run fmt")?;
            if out != 0 {
                anyhow::bail!("fmt failed with exit code {out}");
            }
        }
        Command::Test { no_nextest, pf } => {
            // cargo nextest run --cargo-profile {{ profile }} --all-targets -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib
            let ws = get_workspace_names(false, true).context("Failed to get workspace names")?;
            let mut args = if !no_nextest {
                vec![
                    "nextest",
                    "run",
                    "--cargo-profile",
                    &pf.profile,
                    "--all-targets",
                ]
            } else {
                vec!["test", "--profile", &pf.profile, "--all-targets"]
            };
            if pf.frozen {
                args.push("--frozen");
            }
            args.extend(["-p", "hyprshell"]);
            args.extend(ws.iter().flat_map(|pkg| ["-p", pkg.name.as_str()]));
            info!("Running test/nextest");
            let out = cmd::run_cargo_command(&args, false).context("Failed to run test/nextest")?;
            if out != 0 {
                anyhow::bail!("test/nextest failed with exit code {out}");
            }
        }
    }

    Ok(())
}

const DEP_CRATES_PREFIX: &str = "dep-crates/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WS {
    pub name: String,
    pub path: PathBuf,
}

fn get_workspace_names(
    include_vendored_deps: bool,
    include_xtask: bool,
) -> anyhow::Result<Vec<WS>> {
    let main_cargo =
        load_toml(Path::new("Cargo.toml")).context("Failed to load main Cargo.toml")?;
    let workspace = main_cargo
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|m| {
            m.into_iter()
                .filter_map(|m| m.as_str())
                .filter(|m| include_xtask || !m.ends_with("xtask"))
                .filter(|m| include_vendored_deps || !m.starts_with(DEP_CRATES_PREFIX))
                .map(String::from)
                .filter_map(|m| {
                    let can = fs::canonicalize(Path::new(&m))
                        .inspect_err(|e| {
                            warn!("Failed to canonicalize path for workspace member {m}: {e:?}")
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
    Ok(workspace)
}
