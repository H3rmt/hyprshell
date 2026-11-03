//! Exercise both real bind serializers against a disposable IPC server.
use config_lib::{Modifier, Switch, Windows};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[test]
fn generated_bindings_preserve_lua_and_legacy_release_semantics() {
    if std::env::var_os("HYPRSHELL_BIND_TEST_CHILD").is_some() {
        for modifier in [Modifier::Alt, Modifier::Ctrl, Modifier::Super] {
            let windows = Windows {
                switch: Some(Switch {
                    modifier,
                    key: "F6".into(),
                    ..Switch::default()
                }),
                ..Windows::default()
            };
            for binding in hyprshell_windows_lib::generate_open_keybinds(&windows) {
                exec_lib::binds::apply_exec_bind_lua(&binding).expect("Lua binding");
                exec_lib::binds::apply_exec_bind_legacy(&binding).expect("legacy binding");
            }
        }
        return;
    }
    let directory = std::env::temp_dir().join(format!("hs-bind-{}", std::process::id()));
    let instance = directory.join("hypr/test");
    std::fs::create_dir_all(&instance).expect("test instance");
    let listener = UnixListener::bind(instance.join(".socket.sock")).expect("socket");
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    let done = Arc::new(AtomicBool::new(false));
    let stop = done.clone();
    let server = std::thread::spawn(move || {
        let mut requests = Vec::new();
        while !stop.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("timeout");
                    let mut bytes = [0; 4096];
                    let size = stream.read(&mut bytes).expect("request");
                    requests.push(String::from_utf8(bytes[..size].to_vec()).expect("UTF-8"));
                    stream.write_all(b"ok").expect("response");
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("accept: {error}"),
            }
        }
        requests
    });
    // Set the socket environment only for a child process, without mutating
    // this multi-threaded test process's environment.
    let status = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "generated_bindings_preserve_lua_and_legacy_release_semantics",
        ])
        .env("HYPRSHELL_BIND_TEST_CHILD", "1")
        .env("XDG_RUNTIME_DIR", &directory)
        .env("HYPRLAND_INSTANCE_SIGNATURE", "test")
        .status()
        .expect("child test");
    done.store(true, Ordering::Release);
    let requests = server.join().expect("server");
    std::fs::remove_dir_all(directory).expect("cleanup");
    assert!(status.success());
    for key in [
        "Alt_L",
        "Alt_R",
        "Control_L",
        "Control_R",
        "Super_L",
        "Super_R",
    ] {
        let lua = requests
            .iter()
            .find(|r| r.contains("release = true") && r.contains(key))
            .expect("Lua release");
        assert!(lua.contains("ignore_mods = true"));
        assert!(lua.contains("auto_consuming = true"));
        assert!(lua.contains("transparent = true"));
        let legacy = requests
            .iter()
            .find(|r| r.contains("bindrti") && r.contains(key))
            .expect("legacy release");
        assert!(legacy.contains("CloseSwitch"));
        assert!(!legacy.contains("--event-time"));
    }
    assert!(
        !requests
            .iter()
            .any(|r| r.contains("Shift_L") || r.contains("Shift_R"))
    );
    let lua_open = requests
        .iter()
        .find(|r| r.contains("--event-time") && r.contains("F6"))
        .expect("timestamped Lua open");
    assert!(lua_open.contains("auto_consuming = true"));
    assert!(lua_open.contains("state.pending[state.serial] = state.time"));
}
