use anyhow::Context;
use clap::Parser;
use core_lib::{
    WarnWithDetails, daemon_running, get_default_config_path, get_default_css_path,
    get_default_data_dir,
};
use std::env;

mod cli;
mod data;
mod keybinds;
mod receive_handle;
mod socket;
mod start;
mod util;

mod completions;
#[cfg(feature = "debug_command")]
mod debug;
#[cfg(feature = "debug_command")]
mod default_apps;

fn main() -> anyhow::Result<()> {
    let cli = cli::App::parse();
    let opts = cli.global_opts.clone();

    let level = if opts.quiet {
        "off"
    } else {
        match opts.verbose {
            0 => "info",
            1 => "debug",
            2.. => "trace",
        }
    };
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_target(
            env::var("HYPRSHELL_LOG_MODULE_PATH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
        )
        .with_env_filter(format!(
            "hyprshell={level},config_lib={level},core_lib={level},exec_lib={level},launcher_lib={level},windows_lib={level},hyprland_plugin={level}",
        ))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .unwrap_or_else(|e| tracing::warn!("Unable to initialize logging: {e}"));

    check_features();
    check_env();

    let data_dir = cli.global_opts.data_dir;
    let css_file = cli.global_opts.css_file;
    let config_path = cli.global_opts.config_file;

    match cli.command {
        cli::Command::Run {} => {
            if daemon_running() {
                anyhow::bail!("Daemon already running");
            }
            exec_lib::check_version()
                .warn_details("Unable to check hyprland version, continuing anyway");
            start::start(
                config_path.unwrap_or(get_default_config_path()),
                css_file.unwrap_or(get_default_css_path()),
                data_dir.unwrap_or(get_default_data_dir()),
            )?;
        }
        #[cfg(feature = "generate_config_command")]
        cli::Command::Config { command } => match command {
            cli::ConfigCommand::Generate { force, no_systemd } => {
                use core_lib::Warn;
                let config_path = config_path.unwrap_or(get_default_config_path());
                let css_path = css_file.unwrap_or(get_default_css_path());

                let (override_config, override_css) = config_lib::generate::get_overrides(&force);
                config_lib::generate::check_file_exist(
                    &config_path,
                    &css_path,
                    override_config,
                    override_css,
                )?;

                let (config_data, css_data) = config_lib::generate::prompt_config()?;
                let config = config_lib::generate::generate_config(config_data);
                tracing::trace!("Generated config: {:#?}", config);
                config_lib::write_config(&config_path, &config, override_config).warn();
                config_lib::generate::write_css(&css_path, &css_data, override_css).warn();
                if !no_systemd {
                    config_lib::generate::write_systemd_unit(
                        opts.config_file.as_ref(),
                        opts.css_file.as_ref(),
                        opts.data_dir.as_ref(),
                        &core_lib::get_data_home(),
                    )
                    .warn();
                }
                core_lib::explain_config(&config_path);
            }
            cli::ConfigCommand::Explain {} => {
                core_lib::explain_config(&config_path.unwrap_or(get_default_config_path()));
            }
            cli::ConfigCommand::Check {} => {
                if let Err(err) = config_lib::load_and_migrate_config(
                    &config_path.unwrap_or(get_default_config_path()),
                ) {
                    tracing::warn!("Failed to load config: {err}");
                    std::process::exit(1);
                }
            }
            #[cfg(feature = "config_check_is_default")]
            cli::ConfigCommand::CheckIfDefault {} => {
                if let Ok(config) = config_lib::load_and_migrate_config(
                    &config_path.unwrap_or(get_default_config_path()),
                ) {
                    let config_default = config_lib::Config::default();
                    if config != config_default {
                        tracing::warn!("Current config does not match the default configuration");
                        tracing::info!("Default config: {:#?}", config_default);
                        tracing::info!("Current config: {:#?}", config);
                        std::process::exit(1);
                    } else {
                        tracing::info!("Current config matches the default configuration");
                    }
                }
            }
        },
        #[cfg(feature = "debug_command")]
        cli::Command::Debug { command } => {
            tracing::info!("use with -vv ... to see full logs!");
            match command {
                cli::DebugCommand::ListIcons => {
                    debug::list_icons();
                }
                cli::DebugCommand::ListDesktopFiles => {
                    debug::list_desktop_files();
                }
                cli::DebugCommand::CheckClass { class } => {
                    debug::check_class(class);
                }
                cli::DebugCommand::Search { text, all } => {
                    debug::search(
                        &text,
                        all,
                        &config_path.unwrap_or(get_default_config_path()),
                        &data_dir.unwrap_or(get_default_data_dir()),
                    );
                }
                cli::DebugCommand::DefaultApplications { command } => match command {
                    cli::DefaultApplicationsCommand::Get { mime } => {
                        default_apps::get(&mime).context("unable to get default app")?;
                    }
                    cli::DefaultApplicationsCommand::Add { mime, value } => {
                        default_apps::add(&mime, &value).context("unable to set default app")?;
                    }
                    cli::DefaultApplicationsCommand::List { all } => {
                        default_apps::list(all);
                    }
                    cli::DefaultApplicationsCommand::Check {} => {
                        default_apps::check();
                    }
                },
            };
        }
        cli::Command::Data { command } => match command {
            cli::DataCommand::LaunchHistory { run_cache_weeks } => {
                data::launch_history(
                    run_cache_weeks,
                    &config_path.unwrap_or(get_default_config_path()),
                    &data_dir.unwrap_or(get_default_data_dir()),
                    opts.verbose,
                );
            }
        },
        cli::Command::Completions { shell, base_path } => {
            if let Some(shell) = shell {
                completions::generate(&shell, base_path)
                    .context("Failed to generate completions")?
            } else {
                for shell in ["bash", "fish", "zsh"] {
                    completions::generate(shell, None).context("Failed to generate completions")?
                }
            }
        }
        cli::Command::Socat { json } => core_lib::transfer::send_raw_to_socket(&json)
            .context("Failed to send JSON to socket: is hyprshell running?")?,
    }
    Ok(())
}

fn check_features() {
    tracing::debug!(
        "FEATURES: JSON5 support: {}, Config command: {}, Debug command: {}, Launcher calc: {}",
        cfg!(feature = "json5_config"),
        cfg!(feature = "generate_config_command"),
        cfg!(feature = "debug_command"),
        cfg!(feature = "launcher_calc"),
    );
}

fn check_env() {
    tracing::trace!(
        "ENV: HYPRSHELL_NO_LISTENERS: {}, HYPRSHELL_NO_ALL_ICONS: {}, HYPRSHELL_RELOAD_TIMEOUT: {}, HYPRSHELL_LOG_MODULE_PATH: {}",
        env::var("HYPRSHELL_NO_LISTENERS").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_NO_ALL_ICONS").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_RELOAD_TIMEOUT").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_LOG_MODULE_PATH").unwrap_or_else(|_| "-None-".to_string()),
    )
}
