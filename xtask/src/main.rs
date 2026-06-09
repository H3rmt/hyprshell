use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::env;
use std::fmt::Debug;
use tracing_subscriber::EnvFilter;

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
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Run {},
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
        Command::Run {} => {}
    }

    Ok(())
}
