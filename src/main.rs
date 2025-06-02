use clap::Parser;
use core_lib::{
    Warn, check_version, daemon_running, get_default_config_path, get_default_css_path,
    get_default_data_dir,
};
use std::env;
use std::fs::read_to_string;
use tracing::{debug, info, warn};

mod cli;
mod keybinds;
mod receive;
mod recive_handle;
mod start;

#[cfg(feature = "debug_command")]
mod debug;

fn main() -> anyhow::Result<()> {
    malloc::limit_mmap_threshold();
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
            env::var("LOG_MODULE_PATH")
                .map(|s| s.parse().unwrap_or(false))
                .unwrap_or(false),
        )
        .with_env_filter(format!(
            "hyprshell={level},core_lib={level},exec_lib={level},launcher_lib={level},windows_lib={level}",
        ))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .unwrap_or_else(|e| warn!("Unable to initialize logging: {e}"));

    check_features();

    let config_path = cli
        .global_opts
        .config_file
        .unwrap_or(get_default_config_path());

    let css_path = cli.global_opts.css_file.unwrap_or(get_default_css_path());
    let data_dir = cli.global_opts.data_dir.unwrap_or(get_default_data_dir());

    match cli.command {
        cli::Command::Run {} => {
            if daemon_running() {
                anyhow::bail!("Daemon already running");
            }
            check_version(exec_lib::get_version()).unwrap_or_else(|e| {
                warn!("Unable to check hyprland version, continuing anyway: {e}")
            });
            start::start(config_path, css_path, data_dir)?;
        }
        #[cfg(feature = "generate_config_command")]
        cli::Command::Config { command } => match command {
            cli::ConfigCommand::Generate { force, no_systemd } => {
                use core_lib::Warn;
                let (override_config, override_css) =
                    core_lib::config::generate::get_overrides(&force);
                core_lib::config::generate::check_file_exist(
                    &config_path,
                    &css_path,
                    override_config,
                    override_css,
                )?;

                let (config_data, css_data) = core_lib::config::generate::prompt_config()?;
                let config = core_lib::config::generate::generate_config(config_data);
                core_lib::config::write_config(&config_path, &config, override_config)
                    .warn("create");
                core_lib::config::generate::write_css(&css_path, &css_data, override_css)
                    .warn("create");
                if !no_systemd {
                    core_lib::config::generate::write_systemd_unit(
                        opts.config_file.as_ref(),
                        opts.css_file.as_ref(),
                        opts.data_dir.as_ref(),
                    )
                    .warn("create");
                }
                core_lib::config::explain::check_config(&config_path).warn("check");
            }
            cli::ConfigCommand::Check {} => {
                use core_lib::Warn;
                core_lib::config::explain::check_config(&config_path).warn("check");
            }
        },
        #[cfg(feature = "debug_command")]
        cli::Command::Debug { command } => {
            info!("use with -vv ... to see full logs!");
            match command {
                cli::DebugCommand::CheckClass { class } => {
                    debug::search(class);
                }
                cli::DebugCommand::ListIcons => {
                    debug::list();
                }
                cli::DebugCommand::ListDesktopFiles => {
                    debug::desktop_files();
                }
                cli::DebugCommand::Search { text, all } => {
                    let (plugins, max_items) = core_lib::config::load_config(&config_path)
                        .ok()
                        .and_then(|c| c.launcher.map(|l| (l.plugins, l.max_items)))
                        .unwrap_or((
                            core_lib::config::Plugins {
                                applications: Default::default(),
                                shell: None,
                                terminal: None,
                                websearch: None,
                                calc: None,
                            },
                            5,
                        ));
                    launcher_lib::debug::get_matches(&plugins, &text, all, max_items, &data_dir);
                }
            };
        }
        cli::Command::Data { command } => match command {
            cli::DataCommand::LaunchHistory { run_cache_weeks } => {
                let run_cache_weeks = run_cache_weeks.unwrap_or_else(|| {
                    core_lib::config::load_config(&config_path)
                        .ok()
                        .and_then(|c| {
                            c.launcher.and_then(|l| {
                                l.plugins.applications.as_ref().map(|a| a.run_cache_weeks)
                            })
                        })
                        .unwrap_or(4)
                });

                let runs = launcher_lib::get_applications_stored_runs(run_cache_weeks, &data_dir);

                for (path, run) in runs {
                    // ignore the ini parser for this, just read the file and find, is faster
                    if let Ok(content) = read_to_string(&path) {
                        if let Some(name_line) = content.lines().find(|l| l.starts_with("Name=")) {
                            let name = name_line.trim_start_matches("Name=");
                            // check if verbosity is set, if so, print the name
                            if opts.verbose > 0 {
                                info!("{}: {name} ({run})", path.display());
                            } else if opts.verbose == 0 {
                                info!("{}: {run}", name);
                            }
                        } else {
                            info!("{}: {run}", path.display());
                        }
                    } else {
                        info!("{}: {run}", path.display());
                    }
                }
            }
        },
        cli::Command::Socat { json } => {
            core_lib::send_raw_to_socket(&json).warn("send failed");
        }
    }
    Ok(())
}

pub mod malloc {
    use std::os::raw::c_int;
    const M_MMAP_THRESHOLD: c_int = -3;

    unsafe extern "C" {
        fn mallopt(param: c_int, value: c_int) -> c_int;
    }

    /// Prevents glibc from hoarding memory via memory fragmentation.
    pub fn limit_mmap_threshold() {
        unsafe {
            mallopt(M_MMAP_THRESHOLD, 65536);
        }
    }
}

fn check_features() {
    debug!(
        "FEATURES: TOML support: {}, Bar: {}, Config command: {}, Debug command: {}, Launcher calc: {}",
        cfg!(feature = "toml_config"),
        cfg!(feature = "bar"),
        cfg!(feature = "generate_config_command"),
        cfg!(feature = "debug_command"),
        cfg!(feature = "launcher_calc"),
    );
}
