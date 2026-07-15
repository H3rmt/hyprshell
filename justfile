set default-list
set minimum-version := "1.46.0"

# Run all checks
[group('security')]
all-checks: (check "1") (lint "0") (test "0") audit outdated shear bloat

# Checking for vulnerabilities in dependencies with cargo audit.
[group('security')]
audit: (build "dev") # refresh Cargo.lock
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
    echo "Checking for vulnerabilities in dependencies with cargo audit..."
    cargo audit

# Checking for outdated dependencies using cargo-outdated.
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

# Checking for unused dependencies with cargo shear.
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

# Checking for bloat in binary with cargo bloat.
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

_build-xtask:
    cargo build --package hyprshell-xtask -q


[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
xtask rebuild="0" *args="":
    #!/usr/bin/env bash
    if [ ! -x ./target/debug/hyprshell-xtask ] || [ "{{ rebuild }}" = "1" ]; then
        cargo build --package hyprshell-xtask
    fi
    ./target/debug/hyprshell-xtask -v cmd {{ args }}

# Run clippy on all workspace members, and optionally with --frozen.
[group('checks')]
[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
check rebuild="0": (xtask rebuild "check")

# Run clippy with --fix on all workspace members, and optionally with --frozen. This will attempt to automatically fix any clippy warnings, but may not be able to fix all of them.
[group('dev')]
[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
fix rebuild="0": (xtask rebuild "fix")

# Run formatting checks on all workspace members, and optionally with --frozen.
[group('checks')]
[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
lint rebuild="0": (xtask rebuild "lint")

# Run tests on all workspace members, and optionally with --frozen. By default, tests are run with cargo nextest, but can be disabled with --no-nextest.
[group('checks')]
[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
test rebuild="0": (xtask rebuild "test")

# Format all workspace members.
[group('dev')]
[group('xtask')]
[arg("rebuild", long="rebuild", short = "r", value="1", help="Force rebuild of the xtask binary")]
format rebuild="0": (xtask rebuild "format")

# Build the project with the specified profile.
[group('dev')]
[arg("profile", long="profile", short = "p", help="The rustc profile to use like 'dev' or 'release'")]
build profile="dev" *args:
    cargo build --profile {{ profile }} {{ args }}

# Run the hyprshell with the specified profile and pass any additional arguments. (just run -p release -- run -vv)
[group('dev')]
[arg("profile", long="profile", help="The rustc profile to use like 'dev' or 'release'")]
run profile="dev" *args:
    cargo run --profile {{ profile }} -- {{ args }}
