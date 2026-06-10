use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::env;
use std::fmt::Debug;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::load::load_toml;

mod edit;
mod load;

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
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    SetVersion {
        /// Version to set (e.g., "1.2.3")
        version: String,
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

    match cli.command {
        Command::SetVersion { version } => {
            // Parse version into semver
            let version = semver::Version::parse(&version).context("Failed to parse version")?;
            debug!("running in directory: {}", env::current_dir()?.display());
            let main_cargo = load_toml("Cargo.toml").context("Failed to load main Cargo.toml")?;
            let workspace = main_cargo
                .get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
                .context("Failed to get workspace members from main Cargo.toml")?;
            debug!("workspace members: {workspace}");
        }
    }

    Ok(())
}
