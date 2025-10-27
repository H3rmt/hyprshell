mod boot;
mod exec;
mod helpers;
mod path;

pub use boot::*;
pub use exec::*;
pub use helpers::*;
pub use path::*;
use std::env::{var, var_os};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use tracing::debug;

#[must_use]
pub fn get_daemon_socket_path_buff() -> PathBuf {
    #[allow(clippy::option_if_let_else)]
    let mut buf = if let Some(runtime_path) = var_os("XDG_RUNTIME_DIR") {
        if let Some(instance) = var_os("HYPRLAND_INSTANCE_SIGNATURE") {
            PathBuf::from(runtime_path).join("hypr").join(instance)
        } else {
            PathBuf::from(runtime_path)
        }
    } else if let Ok(uid) = var("UID") {
        PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        PathBuf::from("/tmp")
    };
    buf.push("hyprshell.sock");
    buf
}

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_daemon_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running on {}", buf.display());
        UnixStream::connect(buf).is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

#[allow(clippy::print_stderr, clippy::print_stdout)]
pub fn explain_config(config_path: &Path, add_how_to_explain_again: bool) {
    let config = match config_lib::load_and_migrate_config(config_path, true) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "\x1b[1m\x1b[31mConfig is invalid ({}):\x1b[0m {err:?}\n",
                config_path.display()
            );
            return;
        }
    };
    let info = config_lib::explain(&config, config_path, true, true);
    println!("{info}");

    if daemon_running() {
        println!("Daemon \x1b[32mrunning\x1b[0m");
    } else {
        eprintln!(
            "Daemon \x1b[31mnot running\x1b[0m, start it with `hyprshell run` or `systemctl --user enable --now hyprshell`"
        );
    }

    if add_how_to_explain_again {
        println!("\nTo explain the config again, run `hyprshell explain`\n");
    }
}
