"""Disposable headless Hyprland, controlled windows and a private Wayland keyboard.

No system service changes, real display connection, or global input injection.
Requires cage, Hyprland/hyprctl, and the two build-helpers.sh outputs.
"""

import contextlib
import json
import os
import select
import signal
import socket
import subprocess
import tempfile
import threading
import time
from pathlib import Path

from proxy import Proxy


def wait(predicate, message, timeout=5):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if predicate():
            return
        time.sleep(0.01)
    raise AssertionError(message)


class Session:
    def __init__(
        self,
        binary,
        helpers,
        output,
        legacy=False,
        modifier="alt",
        key="Tab",
        workspaces=False,
    ):
        self.binary = str(Path(binary).resolve())
        self.helpers = Path(helpers).resolve()
        self.output = Path(output).resolve()
        self.output.mkdir(parents=True, exist_ok=True)
        self.legacy, self.modifier, self.switch_key, self.workspaces = (
            legacy,
            modifier,
            key,
            workspaces,
        )
        self.processes, self.logs, self.held = [], [], set()
        self.temp = tempfile.TemporaryDirectory(prefix="hs-", dir="/tmp")
        self.root = Path(self.temp.name)
        self.env = dict(
            os.environ,
            XDG_RUNTIME_DIR=str(self.root),
            XDG_CONFIG_HOME=str(self.root / "config"),
            XDG_DATA_HOME=str(self.root / "data"),
            XDG_CACHE_HOME=str(self.root / "cache"),
            XDG_STATE_HOME=str(self.root / "state"),
            DBUS_SESSION_BUS_ADDRESS="unix:path=" + str(self.root / "no-bus"),
            HYPRLAND_NO_SD_VARS="1",
            HYPRLAND_NO_SD_TARGET="1",
            HYPRLAND_NO_SD_NOTIFY="1",
            RUST_LOG="hyprshell=trace",
            HYPRSHELL_NO_LISTENERS="1",
            HYPRSHELL_NO_ALL_ICONS="1",
            __EGL_VENDOR_LIBRARY_FILENAMES="/run/opengl-driver/share/glvnd/egl_vendor.d/50_mesa.json",
        )
        self.env.pop("HYPRLAND_INSTANCE_SIGNATURE", None)
        self.env.pop("WAYLAND_DISPLAY", None)
        self.env.pop("DISPLAY", None)

    def spawn(self, name, args, env=None, **kwargs):
        log = open(self.output / (name + ".log"), "w")  # noqa: SIM115 - closed in __exit__
        self.logs.append(log)
        p = subprocess.Popen(
            args,
            env=env or self.env,
            stdout=log,
            stderr=log,
            start_new_session=True,
            **kwargs,
        )
        self.processes.append(p)
        return p

    def raw(self, *args):
        return subprocess.check_output(
            ["hyprctl", *args], env=self.env, text=True, timeout=3
        ).strip()

    def hypr(self, command):
        return json.loads(self.raw("-j", command))

    def visible(self):
        return any(
            "hyprshell" in layer["namespace"]
            for monitor in self.hypr("layers").values()
            for level in monitor["levels"].values()
            for layer in level
        )

    def active(self):
        return self.hypr("activewindow").get("address")

    def order(self):
        return [
            c["address"]
            for c in sorted(self.hypr("clients"), key=lambda c: c["focusHistoryID"])
        ]

    def focus(self, address):
        if self.legacy:
            self.raw("dispatch", "focuswindow", "address:" + address)
        else:
            self.raw(
                "dispatch",
                "hl.dsp.focus({window=" + json.dumps("address:" + address) + "})",
            )
        wait(lambda: self.active() == address, "Could not focus controlled window")
        time.sleep(0.06)

    def ipc(self, message):
        subprocess.run(
            [self.binary, "socat", json.dumps(message)],
            env=self.env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
            timeout=3,
        )

    def key(self, code, pressed):
        self.keyboard.stdin.write(f"{code} {pressed}\n")
        self.keyboard.stdin.flush()
        if not select.select([self.keyboard.stdout], [], [], 2)[0]:
            raise AssertionError("Virtual keyboard acknowledgement timed out")
        assert self.keyboard.stdout.readline().strip() == "ok"
        if pressed:
            self.held.add(code)
        else:
            self.held.discard(code)

    def tap(self, code=15, duration=0.02):
        self.key(code, 1)
        time.sleep(duration)
        self.key(code, 0)

    def expect(self, address, label):
        wait(lambda: self.active() == address and not self.visible(), label, 1.5)
        print("PASS:", label, flush=True)

    def __enter__(self):
        try:
            parent = self.root / "parent"
            parent.mkdir()
            cage_env = dict(
                self.env,
                XDG_RUNTIME_DIR=str(parent),
                WLR_BACKENDS="headless",
                WLR_RENDERER="gles2",
                WLR_RENDER_DRM_DEVICE="/dev/dri/renderD128",
            )
            self.spawn(
                "cage",
                [os.environ.get("CAGE", "cage"), "--", "sleep", "3600"],
                cage_env,
            )
            wait(
                lambda: any(p.is_socket() for p in parent.glob("wayland-*")),
                "Headless parent failed",
                15,
            )
            self.env["WAYLAND_DISPLAY"] = str(
                next(p for p in parent.glob("wayland-*") if p.is_socket())
            )
            config = self.root / ("test.conf" if self.legacy else "test.lua")
            config.write_text(
                "monitor = , 1280x720@60, 0x0, 1\ninput {\n repeat_rate = 0\n}\nanimations {\n enabled = false\n}\n"
                if self.legacy
                else 'hl.config({input={repeat_rate=0},animations={enabled=false},misc={disable_watchdog_warning=true},debug={disable_logs=false}})\nhl.monitor({output="",mode="1280x720@60",position="0x0",scale=1})\n'
            )
            hypr = self.spawn(
                "hyprland",
                [os.environ["TEST_HYPRLAND"], "--config", str(config)],
                dict(self.env, AQ_DRM_DEVICES="/dev/dri/renderD128"),
            )

            def instance():
                for d in (self.root / "hypr").glob("*"):
                    if (d / ".socket.sock").is_socket():
                        self.env["HYPRLAND_INSTANCE_SIGNATURE"] = d.name
                        return True
                assert hypr.poll() is None, "Compositor exited"
                return False

            wait(instance, "Disposable compositor IPC failed", 20)

            def output():
                try:
                    response = self.raw("output", "create", "headless", "HEADLESS-TEST")
                    return response in {"ok", "Name already taken"}
                except subprocess.TimeoutExpired:
                    return False

            wait(output, "Could not create headless output", 40)
            wait(lambda: len(self.hypr("monitors")) > 0, "No test output", 10)
            assert all(
                m["name"].startswith(("HEADLESS-", "WAYLAND-"))
                for m in self.hypr("monitors")
            )
            instances = json.loads(self.raw("instances", "-j"))
            self.env["WAYLAND_DISPLAY"] = next(
                x["wl_socket"] for x in instances if x["pid"] == hypr.pid
            )
            errors = self.raw("configerrors")
            if errors.strip():
                raise AssertionError(
                    "Disposable compositor rejected the requested configuration: "
                    + errors
                )
            config_dir = self.root / "config/hyprshell"
            config_dir.mkdir(parents=True)
            (config_dir / "config.json").write_text(
                json.dumps(
                    {
                        "version": 4,
                        "windows": {
                            "overview": None,
                            "switch": {
                                "modifier": self.modifier,
                                "key": self.switch_key,
                                "filter_by": [],
                                "switch_workspaces": self.workspaces,
                            },
                        },
                    }
                )
            )
            self.spawn("windows", [str(self.helpers / "windows")])
            wait(
                lambda: len(self.hypr("clients")) == 3,
                "Controlled windows did not map",
                15,
            )
            # Put one controlled client on each workspace; focus history differs from workspace order.
            for i, address in enumerate(self.order(), 1):
                if self.legacy:
                    self.raw(
                        "dispatch", "movetoworkspacesilent", f"{i},address:{address}"
                    )
                else:
                    self.raw(
                        "dispatch",
                        f'hl.dsp.window.move({{workspace="{i}",window="address:{address}",follow=false}})',
                    )
            for address in self.order()[::-1]:
                self.focus(address)
            self.original = self.active()
            log = open(self.output / "keyboard.log", "w")
            self.logs.append(log)
            self.keyboard = subprocess.Popen(
                [str(self.helpers / "keyboard")],
                env=self.env,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=log,
                text=True,
                bufsize=1,
                start_new_session=True,
            )
            self.processes.append(self.keyboard)
            assert select.select([self.keyboard.stdout], [], [], 3)[0], (
                "Keyboard did not start"
            )
            assert self.keyboard.stdout.readline().strip() == "ready"
            real = self.root / "hypr" / self.env["HYPRLAND_INSTANCE_SIGNATURE"]
            private = self.root / "hypr/proxy"
            private.mkdir()
            (private / ".socket2.sock").symlink_to(real / ".socket2.sock")
            self.proxy = Proxy(real / ".socket.sock")
            self.listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self.listener.bind(str(private / ".socket.sock"))
            self.listener.listen()
            self.proxy_thread = threading.Thread(
                target=self.proxy.serve, args=(self.listener,), daemon=True
            )
            self.proxy_thread.start()
            (real / "hyprshell.sock").symlink_to(private / "hyprshell.sock")
            self.daemon = self.spawn(
                "daemon",
                [
                    self.binary,
                    "--config-file",
                    str(config_dir / "config.json"),
                    "run",
                    "-vv",
                ],
                dict(self.env, HYPRLAND_INSTANCE_SIGNATURE="proxy"),
            )
            wait(
                lambda: (
                    self.root
                    / "hypr"
                    / self.env["HYPRLAND_INSTANCE_SIGNATURE"]
                    / "hyprshell.sock"
                ).is_socket(),
                "Daemon socket did not start",
                20,
            )
            wait(
                lambda: (
                    "Initializing SwitchRoot"
                    in (self.output / "daemon.log").read_text()
                ),
                "Daemon GUI did not initialize",
                15,
            )
            time.sleep(0.5)
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def __exit__(self, *exc):
        if hasattr(self, "keyboard"):
            for key in list(self.held):
                with contextlib.suppress(Exception):
                    self.key(key, 0)
        if hasattr(self, "original"):
            with contextlib.suppress(Exception):
                self.ipc({"CloseSwitch": {"switch": False}})
                self.focus(self.original)
        import shutil

        for log in (self.root / "hypr").glob("*/hyprland.log"):
            shutil.copyfile(log, self.output / "hyprland-detail.log")
        for p in reversed(self.processes):
            with contextlib.suppress(ProcessLookupError):
                os.killpg(p.pid, signal.SIGTERM)
            try:
                p.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
            # Hyprland may leave a helper child after its leader exits. The
            # group belongs to this spawn even after wait() has reaped it.
            with contextlib.suppress(ProcessLookupError):
                os.killpg(p.pid, signal.SIGKILL)
            p.wait()
        if hasattr(self, "proxy"):
            self.proxy.stop()
            self.listener.close()
            self.proxy_thread.join(timeout=1)
        for log in self.logs:
            log.close()
        self.temp.cleanup()
