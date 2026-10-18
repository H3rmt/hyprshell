//! Exercise the public collection and switching entrypoints against a private IPC socket.
//! A child process owns the default Hyprland instance; no test mutates the parent's environment.
use core_lib::ByFirst;
use hyprshell_exec_lib::{
    collect::collect_hypr_data, get_initial_active, switch::switch_workspace,
};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use std::time::Duration;

fn reference(schema: &str) -> Value {
    match schema {
        "legacy" => json!({"id": 2, "name": "renamed"}),
        "numbered" => json!({"address": "2", "type": "numbered", "name": "renamed"}),
        "special" => json!({"address": "special:work", "type": "special", "name": "renamed"}),
        kind => json!({"address": "work", "type": kind, "name": "renamed"}),
    }
}

fn workspace(reference: &Value) -> Value {
    let mut value = json!({"monitor": "DP-1", "monitorID": 0, "windows": 1,
        "hasfullscreen": false, "lastwindow": "0x1234", "lastwindowtitle": "fixture"});
    value
        .as_object_mut()
        .expect("valid test fixture")
        .extend(reference.as_object().expect("valid test fixture").clone());
    value
}

fn monitor(schema: &str) -> Value {
    let active = if schema == "legacy" {
        json!({"id": 1, "name": "1"})
    } else {
        json!({"address": "1", "id": 1, "type": "numbered", "name": "1"})
    };
    json!({"id": 0, "name": "DP-1", "description": "fixture", "width": 1920,
        "height": 1080, "refreshRate": 60.0, "x": 0, "y": 0,
        "activeWorkspace": active, "specialWorkspace": {"address": "", "name": "", "type": ""},
        "reserved": [0,0,0,0], "scale": 1.0, "transform": 0, "focused": true,
        "dpmsStatus": true, "vrr": false, "disabled": false})
}

fn client(reference: &Value) -> Value {
    json!({"address": "0x1234", "mapped": true, "at": [0,0], "size": [200,200],
        "workspace": reference, "floating": false, "monitor": 0,
        "class": "fixture", "title": "fixture", "initialClass": "fixture",
        "initialTitle": "fixture", "pid": 1, "xwayland": false, "pinned": false,
        "fullscreen": 0, "fullscreenClient": 0, "grouped": [], "swallowing": null,
        "focusHistoryID": 0})
}

#[test]
fn workspace_collection_and_dispatch_use_stable_addresses() {
    for schema in ["legacy", "numbered", "named", "normal", "special"] {
        for legacy in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "hyprshell-ws-{}-{schema}-{legacy}",
                std::process::id()
            ));
            let socket_dir = root.join("hypr/fixture");
            std::fs::create_dir_all(&socket_dir).expect("valid test fixture");
            let socket = socket_dir.join(".socket.sock");
            let listener = UnixListener::bind(&socket).expect("valid test fixture");
            let server = std::thread::spawn(move || {
                let reference = reference(schema);
                let monitor = monitor(schema);
                let workspace = workspace(&reference);
                let client = client(&reference);
                let mut dispatches = Vec::new();
                let mut expired = false;
                for connection in listener.incoming() {
                    let mut stream = connection.expect("valid test fixture");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .expect("valid test fixture");
                    stream
                        .set_write_timeout(Some(Duration::from_secs(5)))
                        .expect("valid test fixture");
                    let mut buf = [0; 8192];
                    let count = stream.read(&mut buf).expect("valid test fixture");
                    let command = std::str::from_utf8(&buf[..count]).expect("valid test fixture");
                    let response = match command {
                        "stop" => break,
                        "expire" => {
                            expired = true;
                            "ok".to_owned()
                        }
                        "j/monitors" | "j/monitors all" => json!([monitor]).to_string(),
                        "j/activeworkspace" => super_workspace(&monitor).to_string(),
                        "j/workspaces" if expired => "[]".to_owned(),
                        "j/workspaces" => json!([workspace]).to_string(),
                        "j/clients" => {
                            let mut unmapped = client.clone();
                            unmapped["workspace"] = json!({"address": "", "type": "", "name": ""});
                            json!([client, unmapped]).to_string()
                        }
                        "j/activewindow" => client.to_string(),
                        _ if command.starts_with("/eval ")
                            || command.starts_with("j/dispatch ") =>
                        {
                            dispatches.push(command.to_owned());
                            if legacy && command.starts_with("/eval ") {
                                "eval is only supported with the lua config manager".to_owned()
                            } else {
                                "ok".to_owned()
                            }
                        }
                        _ => panic!("unexpected IPC command: {command}"),
                    };
                    stream
                        .write_all(response.as_bytes())
                        .expect("valid test fixture");
                }
                dispatches
            });
            let result = Command::new(std::env::current_exe().expect("valid test fixture"))
                .args([
                    "--exact",
                    "workspace_fixture_child",
                    "--ignored",
                    "--nocapture",
                ])
                .env("XDG_RUNTIME_DIR", &root)
                .env("HYPRLAND_INSTANCE_SIGNATURE", "fixture")
                .env("HYPRSHELL_WORKSPACE_FIXTURE", schema)
                .env("HYPRSHELL_WORKSPACE_SOCKET", &socket)
                .output()
                .expect("valid test fixture");
            UnixStream::connect(&socket)
                .expect("valid test fixture")
                .write_all(b"stop")
                .expect("valid test fixture");
            let dispatches = server.join().expect("valid test fixture");
            std::fs::remove_dir_all(&root).expect("valid test fixture");
            assert!(
                result.status.success(),
                "{schema}/{legacy}: {}\n{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            assert_dispatches(schema, legacy, &dispatches);
        }
    }
}

fn assert_dispatches(schema: &str, legacy: bool, dispatches: &[String]) {
    assert_eq!(
        dispatches.len(),
        (if legacy { 2 } else { 1 }) * (if schema == "special" { 2 } else { 1 }),
        "{dispatches:?}"
    );
    let selector = match schema {
        "legacy" | "numbered" => "2",
        "special" => "special:work",
        _ => "name:work",
    };
    assert!(dispatches[0].contains(selector), "{dispatches:?}");
    assert!(
        dispatches.iter().all(|d| !d.contains("renamed")),
        "{dispatches:?}"
    );
    if legacy {
        let expected = if schema == "special" {
            "j/dispatch togglespecialworkspace work".to_owned()
        } else {
            format!("j/dispatch workspace {selector}")
        };
        assert_eq!(dispatches[1], expected);
    }
}

fn super_workspace(monitor: &Value) -> Value {
    workspace(&monitor["activeWorkspace"])
}

#[test]
#[ignore = "runs in a child with a private Hyprland IPC socket"]
fn workspace_fixture_child() {
    let schema = std::env::var("HYPRSHELL_WORKSPACE_FIXTURE").expect("valid test fixture");
    let (clients, workspaces, _, _, active, _) =
        collect_hypr_data(None).expect("valid test fixture");
    assert_eq!(clients.len(), 1, "valid clients must survive collection");
    assert_eq!(workspaces.len(), 1);
    let key = workspaces[0].0;
    assert_eq!(clients[0].1.workspace, key);
    assert_eq!(active, key);
    assert_eq!(
        get_initial_active().expect("valid test fixture").workspace,
        1
    );
    assert_eq!(
        workspaces
            .find_by_first(&key)
            .expect("valid test fixture")
            .name,
        "renamed"
    );
    switch_workspace(key).expect("valid test fixture");
    if schema == "special" {
        switch_workspace(1).expect("valid test fixture"); // Deactivate by stable address even after display rename.
    }
    if !matches!(schema.as_str(), "legacy" | "numbered") {
        // The same selection must fail before any dispatcher if the workspace disappears.
        let mut stream = UnixStream::connect(
            std::env::var("HYPRSHELL_WORKSPACE_SOCKET").expect("valid test fixture"),
        )
        .expect("valid test fixture");
        stream.write_all(b"expire").expect("valid test fixture");
        let mut reply = String::new();
        stream
            .read_to_string(&mut reply)
            .expect("valid test fixture");
        assert_eq!(reply, "ok");
        assert!(
            switch_workspace(key)
                .expect_err("invalid test fixture")
                .to_string()
                .contains("no longer exists")
        );
    }
}
