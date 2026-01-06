mod boot;
mod exec;
mod exists;
mod helpers;
mod path;

pub use boot::*;
pub use exec::*;
pub use exists::*;
pub use helpers::*;
pub use path::*;
use std::env::{var, var_os};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
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
