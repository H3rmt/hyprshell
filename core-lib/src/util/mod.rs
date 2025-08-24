mod r#const;
mod exec;
mod helpers;
mod path;

pub use r#const::*;
pub use exec::*;
pub use helpers::*;
pub use path::*;
use std::env::{var, var_os};
use std::path::Path;

pub fn get_daemon_socket_path_buff() -> std::path::PathBuf {
    #[allow(clippy::option_if_let_else)]
    let mut buf = if let Some(runtime_path) = var_os("XDG_RUNTIME_DIR") {
        if let Some(instance) = var_os("HYPRLAND_INSTANCE_SIGNATURE") {
            std::path::PathBuf::from(runtime_path)
                .join("hypr")
                .join(instance)
        } else {
            std::path::PathBuf::from(runtime_path)
        }
    } else if let Ok(uid) = var("UID") {
        std::path::PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        std::path::PathBuf::from("/tmp")
    };
    buf.push("hyprshell.sock");
    buf
}

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_daemon_socket_path_buff();
    if buf.exists() {
        tracing::debug!("Checking if daemon is running");
        std::os::unix::net::UnixStream::connect(buf).is_ok()
    } else {
        tracing::debug!("Daemon not running");
        false
    }
}

pub fn explain_config(config_path: &Path) {
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
    let info = config_lib::explain(&config, config_path);
    println!("{info}");

    if daemon_running() {
        println!("Daemon \x1b[32mrunning\x1b[0m");
    } else {
        eprintln!(
            "Daemon \x1b[31mnot running\x1b[0m, start it with `hyprshell run` or `systemctl --user enable --now hyprshell`"
        );
    }
}
