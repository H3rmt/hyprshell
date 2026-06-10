use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs, process};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::load::load_toml;

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

    /// Dont make any changes, just print what would be done
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Directory to run the command in (defaults to current directory)
    #[arg(long, global = true)]
    pub dir: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    SetVersion {
        /// Version to set (e.g., "1.2.3")
        version: String,
    },
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
}

fn main() -> anyhow::Result<()> {
    let cli = AppArgs::parse();
    let opts = cli.global_opts;

    let level = match opts.verbose {
        0 => "info",
        1 => "debug",
        2.. => "trace",
    };
    tracing_log::LogTracer::init()
        .unwrap_or_else(|e| tracing::warn!("Unable to initialize log logging: {e}"));

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| format!("xtask={level}").into());
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_target(
            env::var("HYPRSHELL_LOG_MODULE_PATH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
        )
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .unwrap_or_else(|e| tracing::warn!("Unable to initialize trace logging: {e}"));

    // Change to the specified directory if provided
    if let Some(dir) = opts.dir {
        debug!("changing directory to: {}", dir);
        env::set_current_dir(&dir).context("Failed to change directory")?;
    } else {
        debug!("running in directory: {}", env::current_dir()?.display());
    }

    match cli.command {
        Command::SetVersion { version } => {
            // Parse version into semver
            let version = semver::Version::parse(&version).context("Failed to parse version")?;
            let main_cargo =
                load_toml(Path::new("Cargo.toml")).context("Failed to load main Cargo.toml")?;
            let workspace = main_cargo
                .get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
                .map(|m| {
                    m.into_iter()
                        .filter_map(|m| m.as_str())
                        .filter_map(|p| fs::canonicalize(Path::new(p)).ok())
                        .collect::<Vec<_>>()
                })
                .context("Failed to get workspace members from main Cargo.toml")?;
            debug!("workspace members: {workspace:?}");
            version::increment_versions(&version, &workspace)
                .context("Failed to increment versions")?;
            version::increment_dependencies(&version, &workspace)
                .context("Failed to increment dependencies")?;
            version::update_lockfile().context("Failed to update Cargo.lock file")?;
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

            // Copy all files and folders inside input_with_path to the temporary directory, preserving their original path.
            for input in input_with_path.iter().flatten() {
                if !input.exists() {
                    anyhow::bail!("Input path does not exist: {}", input.display());
                }
                if input.is_file() {
                    let dest = tmp_path.join(input);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent).context(
                            "Failed to create parent directories for input_with_path file",
                        )?;
                    }
                    fs::copy(input, dest)
                        .context("Failed to copy input_with_path file to temporary directory")?;
                } else if input.is_dir() {
                    copy_dir_all(input, &tmp_path.join(input)).context(
                        "Failed to copy input_with_path directory to temporary directory",
                    )?;
                }
            }
            // Copy all files and folders inside input_flat to the root of the temporary directory, ignoring their original path.
            for input in input_flat.iter().flatten() {
                if !input.exists() {
                    anyhow::bail!("Input path does not exist: {}", input.display());
                }
                if input.is_file() {
                    fs::copy(input, tmp_path.join(input.file_name().unwrap()))
                        .context("Failed to copy input_flat file to temporary directory")?;
                } else if input.is_dir() {
                    copy_dir_all(input, &tmp_path)
                        .context("Failed to copy input_flat directory to temporary directory")?;
                }
            }

            let args = vec![
                "cvfa",
                output.as_str(),
                "-C",
                tmp_path
                    .to_str()
                    .context("Failed to convert temporary path to string")?,
                ".",
            ];
            debug!("running tar with args: {args:?}");
            let cmd = process::Command::new("tar")
                .args(args)
                .output()
                .context("Failed to create tar archive")?;

            if !cmd.status.success() {
                anyhow::bail!(
                    "tar command failed: {}",
                    String::from_utf8_lossy(&cmd.stderr)
                );
            }
            let metadata = fs::metadata(&output).context("Failed to stat output")?;
            info!("created archive {} ({} bytes)", output, metadata.len());

            // remove the temporary directory
            fs::remove_dir_all(&tmp_path).context("Failed to remove temporary directory")?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
