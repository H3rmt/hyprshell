use anyhow::{Context, bail};
use clap::Parser;
use core_lib::WarnWithDetails;
use core_lib::path::{
    get_default_cache_dir, get_default_config_path, get_default_css_path, get_default_data_dir,
};
use core_lib::util::daemon_running;
use std::{env, fs};
use tracing_subscriber::EnvFilter;

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

#[allow(clippy::too_many_lines)]
fn main() -> anyhow::Result<()> {
    let _ = format!("{}", 2);
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
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
            "hyprshell={level},config_lib={level},core_lib={level},exec_lib={level},launcher_lib={level},windows_lib={level},hyprland_plugin={level},hyprshell_clipboard_lib={level},hyprshell_config_edit_lib={level}"
        ).into()}
    );
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
        .unwrap_or_else(|e| tracing::warn!("Unable to initialize logging: {e}"));

    check_features();
    check_env();

    let data_dir = cli.global_opts.data_dir;
    let cache_dir = cli.global_opts.cache_dir;
    let css_file = cli.global_opts.css_file;
    let config_path = cli.global_opts.config_file;

    match cli.command {
        cli::Command::Run {} => {
            if daemon_running() {
                bail!("Daemon already running");
            }
            exec_lib::check_version()
                .warn_details("Unable to check hyprland version, continuing anyway");
            // clipboard_lib::store::test_clipboard();

            start::start(
                config_path.unwrap_or_else(get_default_config_path),
                css_file.unwrap_or_else(get_default_css_path),
                data_dir.unwrap_or_else(get_default_data_dir),
                cache_dir.unwrap_or_else(get_default_cache_dir),
            )?;
        }
        cli::Command::Config { command } => match command {
            cli::ConfigCommand::Edit {} => {
                let config_path = config_path.unwrap_or_else(get_default_config_path);
                let css_path = css_file.unwrap_or_else(get_default_css_path);
                config_edit_lib::start(config_path, css_path);
            }
            #[cfg(feature = "generate_config_command")]
            cli::ConfigCommand::Generate { force, no_systemd } => {
                use core_lib::Warn;
                let config_path = config_path.unwrap_or_else(get_default_config_path);
                let css_path = css_file.unwrap_or_else(get_default_css_path);

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
                if cfg!(debug_assertions) || no_systemd {
                    config_lib::generate::write_systemd_unit(
                        opts.config_file.as_ref(),
                        opts.css_file.as_ref(),
                        opts.data_dir.as_ref(),
                        opts.cache_dir.as_ref(),
                        &core_lib::path::get_data_home(),
                    )
                    .warn();
                }
                core_lib::util::explain_config(&config_path, true);
            }
            cli::ConfigCommand::Explain {} => {
                core_lib::util::explain_config(
                    &config_path.unwrap_or_else(get_default_config_path),
                    false,
                );
            }
            cli::ConfigCommand::Check {} => {
                if let Err(err) = config_lib::load_and_migrate_config(
                    &config_path.unwrap_or_else(get_default_config_path),
                    true,
                ) {
                    tracing::warn!("Failed to load config: {err:?}");
                    std::process::exit(1);
                }
            }
            #[cfg(feature = "ci_config_check")]
            cli::ConfigCommand::CheckIfDefault {} => {
                let config = config_lib::load_and_migrate_config(
                    &config_path.unwrap_or_else(get_default_config_path),
                    false,
                )?;
                let config_default = config_lib::Config::default();
                if config == config_default {
                    tracing::info!("Current config matches the default configuration");
                } else {
                    tracing::warn!("Current config does not match the default configuration");
                    tracing::info!("Default config: {:#?}", config_default);
                    tracing::info!("Current config: {:#?}", config);
                    std::process::exit(1);
                }
            }
            #[cfg(feature = "ci_config_check")]
            cli::ConfigCommand::CheckIfFull {} => {
                let config = config_lib::load_and_migrate_config(
                    &config_path.unwrap_or_else(get_default_config_path),
                    false,
                )?;
                let config_all = config_lib::Config {
                    windows: Some(config_lib::Windows {
                        overview: Some(config_lib::Overview::default()),
                        switch: Some(config_lib::Switch::default()),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                if config == config_all {
                    tracing::info!("Current config matches the default configuration");
                } else {
                    tracing::warn!("Current config does not match the full configuration");
                    tracing::info!("All config: {:#?}", config_all);
                    tracing::info!("Current config: {:#?}", config);
                    std::process::exit(1);
                }
            }
        },
        #[cfg(feature = "debug_command")]
        #[allow(clippy::print_stderr, clippy::print_stdout)]
        cli::Command::Debug { command } => {
            println!("run with -vv ... to see all logs");
            match command {
                cli::DebugCommand::ListIcons => {
                    debug::list_icons().warn_details("Failed to list icons");
                }
                cli::DebugCommand::ListDesktopFiles => {
                    debug::list_desktop_files();
                }
                cli::DebugCommand::CheckClass { class } => {
                    debug::check_class(class).warn_details("Failed to check class");
                }
                cli::DebugCommand::Search { text, all } => {
                    debug::search(
                        &text,
                        all,
                        &config_path.unwrap_or_else(get_default_config_path),
                        &data_dir.unwrap_or_else(get_default_data_dir),
                    );
                }
                cli::DebugCommand::DefaultApplications { command } => match command {
                    cli::DefaultApplicationsCommand::Get { mime } => {
                        default_apps::get(&mime).context("unable to get default app")?;
                    }
                    cli::DefaultApplicationsCommand::Set { mime, value } => {
                        default_apps::set_default(&mime, &value)
                            .context("unable to set default app")?;
                    }
                    cli::DefaultApplicationsCommand::Add { mime, value } => {
                        default_apps::add_association(&mime, &value)
                            .context("unable to add association")?;
                    }
                    cli::DefaultApplicationsCommand::List { all } => {
                        default_apps::list(all);
                    }
                    cli::DefaultApplicationsCommand::Check {} => {
                        default_apps::check();
                    }
                },
            }
        }
        cli::Command::Data { command } => match command {
            cli::DataCommand::LaunchHistory { run_cache_weeks } => {
                data::launch_history(
                    run_cache_weeks,
                    &config_path.unwrap_or_else(get_default_config_path),
                    &data_dir.unwrap_or_else(get_default_data_dir),
                    opts.verbose,
                );
            }
        },
        cli::Command::Completions {
            shell,
            base_path,
            delete,
        } => {
            if let Some(shell) = shell {
                completions::generate(&shell, base_path, delete)
                    .context("Failed to generate completions")?;
            } else {
                for shell in ["bash", "fish", "zsh"] {
                    completions::generate(shell, None, delete)
                        .context("Failed to generate completions")?;
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
        "FEATURES: json5_config: {}, generate_config_command: {}, debug_command: {}, launcher_calc: {}, clipboard_compress_lz4: {}, clipboard_compress_zstd: {}, clipboard_encrypt_chacha20poly1305: {}, clipboard_encrypt_aes_gcm: {}",
        cfg!(feature = "json5_config"),
        cfg!(feature = "generate_config_command"),
        cfg!(feature = "debug_command"),
        cfg!(feature = "launcher_calc"),
        cfg!(feature = "clipboard_compress_lz4"),
        cfg!(feature = "clipboard_compress_zstd"),
        cfg!(feature = "clipboard_encrypt_chacha20poly1305"),
        cfg!(feature = "clipboard_encrypt_aes_gcm"),
    );
}

fn check_env() {
    tracing::debug!(
        "ENV: HYPRSHELL_NO_LISTENERS: {}, HYPRSHELL_NO_ALL_ICONS: {}, HYPRSHELL_RELOAD_TIMEOUT: {}, HYPRSHELL_LOG_MODULE_PATH: {}, HYPRSHELL_NO_USE_PLUGIN: {}, HYPRSHELL_EXPERIMENTAL: {}, HYPRSHELL_RUN_ACTIONS_IN_DEBUG: {}",
        env::var("HYPRSHELL_NO_LISTENERS").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_NO_ALL_ICONS").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_RELOAD_TIMEOUT").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_LOG_MODULE_PATH").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_NO_USE_PLUGIN").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_EXPERIMENTAL").unwrap_or_else(|_| "-None-".to_string()),
        env::var("HYPRSHELL_RUN_ACTIONS_IN_DEBUG").unwrap_or_else(|_| "-None-".to_string()),
    );
    let os_name = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("NAME="))
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "NAME=Unknown".to_string());

    tracing::debug!(
        "OS: {}, ARCH: {}, {}",
        env::consts::OS,
        env::consts::ARCH,
        os_name,
    );
}
