use clap::{Args, Parser, Subcommand};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about,
    long_about = "A modern GTK4-based window switcher and application launcher for Hyprland"
)]
pub struct App {
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

    /// Turn off all output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Path to config [default: `$XDG_CONFIG_HOME/hyprshell/config.ron`],
    /// allowed file types: ron, toml, json5
    #[arg(short = 'c', long, global = true)]
    pub config_file: Option<PathBuf>,

    /// Path to css [default: `$XDG_CONFIG_HOME/hyprshell/styles.css`]
    #[arg(long, short = 's', global = true)]
    pub css_file: Option<PathBuf>,

    /// Path to data directory [default: `$XDG_DATA_HOME/hyprshell`]
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    /// Path to cache directory [default: `$XDG_CACHE_HOME/hyprshell`]
    #[arg(long, global = true)]
    pub cache_dir: Option<PathBuf>,

    /// TODO no longer used, replaced by XDG_DATA_DIRS
    /// Path to system data directory [default: `/usr/share/hyprshell`]
    #[arg(long, global = true)]
    pub system_data_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Run the hyprshell daemon
    Run {},

    /// Generate or check the config file
    Config {
        #[clap(subcommand)]
        command: ConfigCommand,
    },

    /// Debug command to debug finding icons for the GUI, desktop files, etc.
    Debug {
        #[clap(subcommand)]
        command: DebugCommand,
    },

    /// Show data, like launch history, etc.
    Data {
        #[clap(subcommand)]
        command: DataCommand,
    },

    /// Send json to the hyprshell socket
    #[clap(hide = true)]
    Socat {
        /// JSON to send to the socket
        json: String,
    },

    /// Internal helper commands
    #[clap(hide = true)]
    Internal {
        #[clap(subcommand)]
        command: InternalCommand,
    },

    /// Generate completions for shells
    Completions {
        /// Shell to generate completion for (if not set completions for all shells will be generated)
        shell: Option<String>,

        /// BASE Path for completion without filename
        /// Bash Default: `/usr/share/bash-completion/completions`
        /// Fish Default: `/usr/share/fish/vendor_completions.d`
        /// Zsh Default: `/usr/share/zsh/site-functions`
        #[arg(long, short = 'p')]
        base_path: Option<PathBuf>,

        /// Delete the generated completion files
        #[arg(short = 'd', long, default_value_t = false)]
        delete: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum InternalCommand {
    /// Send JSON to the hyprshell socket
    Socat {
        /// JSON to send to the socket
        json: String,
    },

    /// Run a command detached from the launcher
    Run(InternalRunArgs),
}

#[derive(Args, Debug, Clone)]
pub struct InternalRunArgs {
    /// Directory in which command output logs are stored
    #[arg(long, value_name = "PATH", required = true)]
    pub log_dir: PathBuf,

    /// Store command stdout and stderr for every run
    #[arg(long)]
    pub store_logs: bool,

    /// Keep the command in the current process group
    #[arg(long, alias = "same-process-group")]
    pub same_process_group: bool,

    /// Interpret COMMAND as a bash command string
    #[arg(long)]
    pub shell: bool,

    /// Working directory for COMMAND
    #[arg(long, value_name = "PATH")]
    pub working_directory: Option<PathBuf>,

    /// Command to execute; place it after `--`
    #[arg(last = true, required = true, num_args = 1..)]
    pub command: Vec<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    /// Generate a default config file (just opens GUI editor)
    Generate {},

    /// Edit the config file with a GUI
    Edit {},

    /// Check the config file for errors
    Check {},

    /// Explain how to use the program based on the config
    Explain {},

    #[cfg(feature = "ci_config_check")]
    /// Check if the provided config is equal to the default config
    CheckIfDefault {},

    #[cfg(feature = "ci_config_check")]
    /// Check if the provided config is equal to the fully enabled config
    CheckIfFull {},
}

#[derive(Subcommand, Debug, Clone)]
pub enum DataCommand {
    /// Show the history of launched applications
    LaunchHistory {
        /// weeks to include in the history, defaults to set config value
        run_cache_weeks: Option<u8>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DebugCommand {
    /// List all icons in the theme
    ListIcons,

    /// List all desktop files
    ListDesktopFiles,

    /// Search for an icon with a window class
    CheckClass {
        /// The class (from `hyprctl clients -j | jq -e ".[] | {title, class}"`) of a window to find an icon for
        ///
        /// If not provided, all open windows will be searched
        class: Option<String>,
    },

    /// simulate search in launcher and display search insights
    Search {
        /// text entered into the search box
        text: String,

        /// Show all matches, not just x ones like configured in config
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// get or set default applications for different mime types
    DefaultApplications {
        #[clap(subcommand)]
        command: DefaultApplicationsCommand,
    },

    /// print debug info
    Info {},
}

#[derive(Subcommand, Debug, Clone)]
pub enum DefaultApplicationsCommand {
    /// Get default app for mimetype
    Get {
        /// for example `image/png` of `x-scheme-handler/https`
        mime: String,
    },

    /// Sets a default app for a mimetype (if one already exists, it is replaced)
    Set {
        /// for example `image/png` of `x-scheme-handler/https`
        mime: String,
        /// Name of a desktop file (with .desktop extension)
        value: String,
    },

    /// Add an association app for mimetype (if one already exists, this one is placed before)
    Add {
        /// for example `image/png` of `x-scheme-handler/https`
        mime: String,
        /// Name of a desktop file (with .desktop extension)
        value: String,
    },

    /// List default apps for all mimetypes
    List {
        /// Show all mimes instead of ony the ones used by hyprshell
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// Check if all entries in all mimetype files point to valid desktop files
    Check {},
}
