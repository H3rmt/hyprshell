default:
    @just --list --justfile {{ justfile() }}

[group('security')]
audit:
    #!/usr/bin/env bash
    if ! command -v cargo-audit >/dev/null 2>&1; then
        echo "cargo-audit not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-audit
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-outdated
        fi
    fi
    echo "Checking for vulnerabilities with cargo audit..."
    cargo audit

[group('security')]
outdated:
    #!/usr/bin/env bash
    if ! command -v cargo-outdated >/dev/null 2>&1; then
        echo "cargo-outdated not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-outdated
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-outdated
        fi
    fi
    echo "Checking for outdated dependencies with cargo outdated..."
    cargo outdated

[group('security')]
shear:
    #!/usr/bin/env bash
    if ! command -v cargo-shear >/dev/null 2>&1; then
        echo "cargo-shear not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-shear
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-shear
        fi
    fi
    echo "Checking for unused dependencies with cargo shear..."
    cargo shear

[group('security')]
bloat:
    #!/usr/bin/env bash
    if ! command -v cargo-bloat >/dev/null 2>&1; then
        echo "cargo-bloat not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-bloat
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-bloat
        fi
    fi
    echo "Checking for bloat in binary with cargo bloat..."
    cargo bloat --release

# [group('develop')]
# fix:
#     cargo fix --allow-dirty -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib

# [group('checks')]
# check-default-nix-features:
#     nix build '.#checks.x86_64-linux.hyprshell-check-nix-configs' -L

[group('run')]
build profile="dev":
    cargo build --profile {{ profile }}

[group('run')]
run profile="dev" *args="":
    cargo run --profile {{ profile }} -- {{ args }}

[group('run')]
run-run profile="dev" *args="-vv": (run profile "run" args)

[group('run')]
run-edit-config profile="dev" *args="-vv": (run profile "config edit" args)

[group('run')]
run-explain-config profile="dev" *args="-vv": (run profile "config explain" args)

[group('run')]
run-debug profile="dev" *args="": (run profile "debug" args)
