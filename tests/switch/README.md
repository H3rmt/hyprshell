# Switch release reproduction

These tests launch a private headless Cage and nested Hyprland, three controlled
GTK windows, and the supplied Hyprshell executable. They use a private Wayland
virtual keyboard, separate XDG directories, and a disposable IPC proxy. They do
not stop a service or use `/dev/uinput`. Failure cleanup releases synthetic keys,
cancels the selection, restores focus within the disposable session, and stops
only the processes created by the test.

Requirements: Python 3, a Hyprland executable that can run nested (use the actual
binary, not a privileged launcher), `hyprctl`, Cage, and a render node. Building
the helpers requires a C compiler, `wayland-scanner`, pkg-config, GTK4,
libxkbcommon, Wayland client libraries, and
`virtual-keyboard-unstable-v1.xml` from a compositor protocol collection.

```sh
tests/switch/build-helpers.sh /path/to/virtual-keyboard-unstable-v1.xml /tmp/switch-helpers
TEST_HYPRLAND=/path/to/Hyprland CAGE=/path/to/cage \
  python3 tests/switch/check.py target/debug/hyprshell /tmp/switch-helpers /tmp/switch-results
```

The output directory contains compositor, application, and daemon logs. The
private compositor disables automatic key repeat so a loaded host cannot turn a
held-key release check into extra navigation; repeated navigation uses explicit
key presses. A case
fails unless both the intended window has focus and the overlay has disappeared.
Held and cancelled selections check both unchanged focus and overlay visibility.
The timing case runs 200 reproducible chords and checks recent-window selection
on every chord. The concurrent release case sends `switch: true`.

Use `--case baseline` on an unmodified build to reproduce close-before-open.
The default `all` also checks navigation receipt ordering, delayed timestamped
opens after Escape and the next quick chord, configuration reload, socket stalls,
malformed responses, and unsupported state queries. Use `--case basic` with each
combination of `--modifier alt|ctrl|super` and `--workspaces` (omit for windows).
`--key F6` exercises a configured switch key. Set `HYPRSHELL_EXPERIMENTAL=1` to
exercise live thumbnails in a build with the `live_windows` feature.

`--case data-failure` injects two failed initial window-data reads, then checks
that the next quick chord selects the recent window and hides the overlay without
Escape. It also checks that failed opens stop polling and that the successful
retry acknowledges all delivered Lua open IDs. This case is included in `all`.

`--legacy --case basic` requires a compositor which actually accepts legacy
configuration files. The harness checks configuration errors before starting the
daemon. A current Lua-only compositor cannot validate this path. Injecting an
unsupported state query tests Hyprshell's event fallback on that compositor; it
is not a substitute for a live legacy compositor test. In event-only mode the
navigation checks wait for delivery before releasing the modifier: independently
spawned, untagged legacy commands have no receipt or cancellation timestamp
contract. Do not run the Lua-specific receipt case as evidence for legacy mode.

The Rust tests exercise the actual release state machine, GLib source ownership,
and generated Lua/legacy bindings. The vendored IPC framing test is separate:

```sh
cargo test --locked -p hyprshell-windows-lib
cargo test --locked -p hyprshell-hyprland --no-default-features --features ctl,async-lite modifier_tests
```
