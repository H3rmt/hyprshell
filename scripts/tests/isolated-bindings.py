import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def wait_for(predicate, message, processes=()):
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        if any(process.poll() is not None for process in processes):
            raise RuntimeError(f"A test process exited while waiting for {message}")
        try:
            result = predicate()
            if result:
                return result
        except (OSError, subprocess.CalledProcessError, json.JSONDecodeError):
            pass
        time.sleep(0.1)
    raise RuntimeError(f"Timed out waiting for {message}")


def terminate(process):
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def run_inside(arguments):
    root = Path(arguments.root)
    runtime = root / "runtime"
    runtime.mkdir(mode=0o700, exist_ok=True)
    environment = os.environ.copy()
    for name in ["HYPRLAND_INSTANCE_SIGNATURE", "WAYLAND_DISPLAY", "DISPLAY", "HYPRSHELL_NO_LISTENERS"]:
        environment.pop(name, None)
    environment.update({
        "XDG_RUNTIME_DIR": str(runtime),
        "XDG_CONFIG_HOME": str(root / "config"),
        "XDG_DATA_HOME": str(root / "data"),
        "XDG_CACHE_HOME": str(root / "cache"),
        "HYPRLAND_NO_SD_VARS": "1",
        "HYPRLAND_NO_SD_NOTIFY": "1",
        "HYPRLAND_NO_CRASHREPORTER": "1",
        "HYPRSHELL_RELOAD_DELAY": "50",
        "HYPRSHELL_RELOAD_DEBOUNCE": "50",
        "GDK_BACKEND": "wayland",
        "GTK_USE_PORTAL": "0",
        "GIO_USE_VFS": "local",
    })
    if arguments.weston_root:
        libraries = Path(arguments.weston_root) / "usr/lib"
        environment["LD_LIBRARY_PATH"] = f"{libraries}:{libraries / 'weston'}"
        environment["WESTON_MODULE_MAP"] = ";".join(
            f"{module.name}={module}" for module in libraries.glob("**/*.so")
        )
        environment["WESTON_DATA_DIR"] = str(Path(arguments.weston_root) / "usr/share/weston")
    if arguments.library_dir:
        environment["LD_LIBRARY_PATH"] = f"{arguments.library_dir}:{environment.get('LD_LIBRARY_PATH', '')}"

    weston = compositor = daemon = None
    handles = []

    def launch(command, name, env):
        output = (root / f"{name}.log").open("w")
        handles.append(output)
        return subprocess.Popen(command, env=env, stdout=output, stderr=subprocess.STDOUT)

    def ipc(*command):
        return subprocess.check_output(["hyprctl", *command], env=environment, text=True, stderr=subprocess.STDOUT)

    def table():
        return json.loads(ipc("-j", "binds"))

    def switch_bindings():
        return [binding for binding in table() if binding["modmask"] in [4, 5] and binding["key"] in ["Tab", "grave", "F6"]]

    def write_config(key="Tab", reverse=True):
        reverse_keys = json.dumps(["grave"] if reverse else [])
        extra = f"\nkeys = [{json.dumps(key)}]\nreverse_keys = {reverse_keys}" if arguments.configurable else ""
        (root / "switch.toml").write_text(f"version = 4\n[windows.switch]\nmodifier = \"ctrl\"\nkey = {json.dumps(key)}{extra}\n")

    def start_daemon(index):
        return launch([
            arguments.binary, "-vv", "--config-file", str(root / "switch.toml"),
            "--css-file", str(root / "styles.css"), "--data-dir", str(root / "hyprshell-data"),
            "--cache-dir", str(root / "hyprshell-cache"), "run",
        ], f"hyprshell-{index}", environment)

    try:
        weston = launch([
            arguments.weston, "--backend=headless", "--renderer=gl", "--fake-seat",
            "--no-config", "--shell=kiosk-shell.so", "--socket=parent-test", "--idle-time=0",
        ], "weston", environment)
        wait_for(lambda: (runtime / "parent-test").exists(), "headless parent compositor", [weston])
        environment["WAYLAND_DISPLAY"] = "parent-test"
        if arguments.legacy:
            configuration = root / "hyprland.conf"
            configuration.write_text("monitor = ,preferred,auto,1\nxwayland:enabled = false\nbind = SUPER,c,exec,true\n")
        else:
            configuration = root / "hyprland.lua"
            configuration.write_text("hl.config({ xwayland = { enabled = false } })\nhl.bind('SUPER + c', function() end, { description = 'Desktop callback fixture' })\n")
        compositor = launch(["Hyprland", "--config", str(configuration)], "hyprland", environment)
        socket_path = wait_for(lambda: next((runtime / "hypr").glob("*/.socket.sock"), None), "isolated Hyprland socket", [compositor])
        environment["HYPRLAND_INSTANCE_SIGNATURE"] = socket_path.parent.name
        display = wait_for(lambda: next((candidate for candidate in runtime.glob("wayland-*") if candidate.is_socket()), None), "isolated Wayland display", [compositor])
        environment["WAYLAND_DISPLAY"] = display.name
        wait_for(table, "isolated binding table", [compositor])
        if arguments.legacy:
            ipc("keyword", "bind", "SUPER,F10,exec,true")
        else:
            ipc("eval", "hl.bind('SUPER + F10', function() end, { description = 'Runtime-only marker' })")
        baseline = table()
        write_config()
        (root / "styles.css").write_text("")
        daemon = start_daemon(1)
        wait_for(lambda: len(switch_bindings()) == 3, "initial switch bindings", [compositor, daemon])
        time.sleep(1)
        first = table()
        assert all(binding in first for binding in baseline), "Startup changed unrelated bindings or reloaded the compositor"
        terminate(daemon)
        daemon = start_daemon(2)
        wait_for(lambda: len(switch_bindings()) == 3, "second daemon start", [compositor, daemon])
        time.sleep(1)
        assert table() == first, "Restart changed callback references or accumulated bindings"
        if arguments.configurable:
            write_config(reverse=False)
            wait_for(lambda: len(switch_bindings()) == 2, "removed grave shortcut", [compositor, daemon])
            without_grave = table()
            assert not any(binding["modmask"] == 4 and binding["key"] == "grave" for binding in without_grave)
            assert without_grave == [binding for binding in first if not (binding["modmask"] == 4 and binding["key"] == "grave")]
            terminate(daemon)
            daemon = start_daemon(3)
            wait_for(lambda: len(switch_bindings()) == 2, "persistent disabled grave shortcut", [compositor, daemon])
            time.sleep(1)
            assert table() == without_grave, "Disabled shortcut or callback identities did not persist"
        else:
            write_config(key="F6")
            wait_for(lambda: not any(binding["key"] == "Tab" for binding in switch_bindings()), "retired Tab bindings", [compositor, daemon])
            assert all(binding in table() for binding in baseline)
        (root / "result.json").write_text(json.dumps({"legacy": arguments.legacy, "configurable": arguments.configurable, "before": baseline, "after": table()}, indent=2))
        print(f"Isolated {'legacy' if arguments.legacy else 'Lua'} binding integration passed", flush=True)
    finally:
        for process in [daemon, compositor, weston]:
            terminate(process)
        for handle in handles:
            handle.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--weston", default=shutil.which("weston"))
    parser.add_argument("--weston-root")
    parser.add_argument("--library-dir", help="Optional test-only compositor dependency directory")
    parser.add_argument("--render-node", required=True)
    parser.add_argument("--legacy", action="store_true")
    parser.add_argument("--configurable", action="store_true")
    parser.add_argument("--inside", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--root", help=argparse.SUPPRESS)
    arguments = parser.parse_args()
    if arguments.inside:
        if os.environ.get("HYPRSHELL_ISOLATED_TEST") != "1":
            parser.error("The compositor test must run inside its device-isolated sandbox")
        run_inside(arguments)
        return
    if not arguments.weston:
        parser.error("Weston is required; pass --weston (and --weston-root for an unpacked package)")
    binary = str(Path(arguments.binary).resolve(strict=True))
    render_node = Path(arguments.render_node)
    if not render_node.name.startswith("renderD") or render_node.parent != Path("/dev/dri"):
        parser.error("Only a DRM render node may be exposed; never expose card or input devices")
    with tempfile.TemporaryDirectory(prefix="hs-") as temporary:
        (Path(temporary) / "runtime").mkdir(mode=0o700)
        command = [
            "bwrap", "--ro-bind", "/", "/", "--dev", "/dev", "--dir", "/dev/dri",
            "--dev-bind", str(render_node), str(render_node), "--proc", "/proc", "--tmpfs", "/run",
            "--unshare-pid", "--unshare-net", "--unshare-ipc", "--unshare-uts",
            "--die-with-parent", "--tmpfs", "/tmp", "--bind", temporary, temporary,
            "--unsetenv", "HYPRLAND_INSTANCE_SIGNATURE", "--unsetenv", "WAYLAND_DISPLAY",
            "--unsetenv", "DISPLAY", "--unsetenv", "DBUS_SESSION_BUS_ADDRESS",
            "--setenv", "XDG_RUNTIME_DIR", str(Path(temporary) / "runtime"),
            "--setenv", "HYPRSHELL_ISOLATED_TEST", "1",
        ]
        if arguments.weston_root:
            bundle = str(Path(arguments.weston_root).resolve(strict=True))
            command.extend(["--ro-bind", bundle, bundle])
        if arguments.library_dir:
            libraries = str(Path(arguments.library_dir).resolve(strict=True))
            command.extend(["--ro-bind", libraries, libraries])
        command.extend([
            "dbus-run-session", "--", sys.executable,
            str(Path(__file__).resolve()), "--inside", "--root", temporary, "--binary", binary,
            "--weston", str(Path(arguments.weston).resolve(strict=True)),
            "--render-node", str(render_node),
        ])
        if arguments.weston_root:
            command.extend(["--weston-root", str(Path(arguments.weston_root).resolve(strict=True))])
        if arguments.library_dir:
            command.extend(["--library-dir", str(Path(arguments.library_dir).resolve(strict=True))])
        for option in ["legacy", "configurable"]:
            if getattr(arguments, option):
                command.append(f"--{option}")
        try:
            subprocess.run(command, check=True)
        except subprocess.CalledProcessError:
            for log in Path(temporary).glob("*.log"):
                print(f"\n--- {log.name} ---\n{log.read_text()[-10000:]}", file=sys.stderr)
            raise


if __name__ == "__main__":
    main()
