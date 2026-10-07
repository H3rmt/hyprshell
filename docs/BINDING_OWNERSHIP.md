# Binding ownership and daemon restarts

Hyprshell no longer reloads the entire Hyprland configuration at startup. It
reconciles its own overview/switch bindings instead: unchanged bindings are
reused, missing bindings are registered, and removed bindings are retired.
Repeated GUI rebuilds and daemon starts do not accumulate bindings.

With the Lua configuration manager, binding handles and their specifications
are retained in the private `__hyprshell_bindings_v1` compositor registry.
Callback references are opaque identities, not commands. Hyprshell does not
recreate other applications' or desktop callbacks.

With legacy syntax, exact binding fingerprints are saved atomically under
`<hyprshell-cache-dir>/keybinds-v1/<HYPRLAND_INSTANCE_SIGNATURE>.json`. The ledger
is locked during reconciliation. Unknown fields in the binding table are
retained in fingerprints. Removing the ledger while bindings are still present
loses proof of ownership and may produce a conflict; it is not a cleanup method.

Unbinding can affect every binding sharing a chord, including other submaps,
even through handles on some supported Hyprland releases. Hyprshell therefore
refuses to modify a chord when it contains an unowned binding. The error names
the chord. Resolve the conflicting configuration rather than broadly unbinding
or reloading the desktop as a workaround.

Bindings left by older Hyprshell versions have no ownership records. Hyprshell
does not infer ownership from descriptions or guess what Lua references mean.
If an upgrade encounters these bindings, a fresh Hyprland session started by
the user will clear them. Hyprshell never initiates a desktop restart, resets
input devices, or reloads the desktop configuration to perform this migration.

An externally initiated Hyprland reload invalidates old handles; the next
Hyprshell configuration event registers missing bindings. Partial registration
failures stop immediately instead of falling back to a second binding backend.
An interrupted legacy registration that could not record its fingerprint is
reported as unowned on the next attempt, not silently overwritten.

## Tests

Run `cargo test -p hyprshell-exec-lib --lib` for mocked legacy IPC, ownership,
failure, and backend-selection tests. Run `lua scripts/tests/owned-bindings.lua`
for the Lua registry tests, including old handle-removal semantics.

Compositor integration testing must use a separate compositor instance and
temporary configuration, data, and cache directories. Compare `hyprctl -j
binds` before and after repeated starts: unrelated Lua dispatcher references
must remain unchanged. Binding-table validation is not proof of physical
shortcut operation; physical checks remain a user test.

The integration harness requires Python, bubblewrap, dbus-run-session, a
GL-capable headless Weston, Hyprland, and a prebuilt Hyprshell executable:

```sh
python scripts/tests/isolated-bindings.py \
  --binary target/debug/hyprshell --render-node /dev/dri/renderD128
```

Choose an available DRM render node; card and input devices are never exposed.
The host filesystem is read-only, host session sockets are hidden, and PID,
network, IPC, and D-Bus sessions are isolated. The harness does not install a
development binary or send input events. Use `--legacy` for the legacy manager.
Use `--configurable` when also testing the configurable-switch-keys change;
this checks that removing grave preserves Tab, Shift+Tab, and every other
binding across another daemon start.

A compatible nested Wayland backend is required. Some Aquamarine releases
request protocol versions newer than the parent advertises. A temporary,
test-only dependency build can be supplied through `--library-dir` without
installing it; report such a workaround with integration results. Unpacked
Weston packages can use `--weston` and `--weston-root`. If nesting cannot start,
report that limitation rather than running the test on the user's compositor.
