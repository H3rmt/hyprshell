#!/usr/bin/env bash
set -euxo pipefail

# Script to run various (Nix Flake) checks
cd "$(dirname "$(realpath "$0")")/.."

help() {
    echo "Usage: $0 <command>"
    echo "Commands:"
    echo "  build                    - Run build check"
    echo "  build-nix                - Run build check nix command"
    echo "  clippy                   - Run clippy check"
    echo "  clippy-nix               - Run clippy check nix command"
    echo "  fmt                      - Run format check"
    echo "  fmt-nix                  - Run format check nix command"
    echo "  test                     - Run tests"
    echo "  test-nix                 - Run tests nix command"
    echo "  check-nix-configs        - Check Nix configurations nix command"
    echo "  check-features           - Check all feature combinations nix command"
    echo "  all                      - Run all checks"
    echo "  all-nix                  - Run all checks as nix commands"
    exit 1
}

[ $# -eq 0 ] && help

case "$1" in
    "build")
        cargo build --profile dev --locked
        ;;
    "build-nix")
        nix build '.#checks.x86_64-linux.hyprshell-config-check' -L
        ;;
    "clippy")
        cargo clippy --profile dev --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib -- --deny warnings
        ;;
    "clippy-nix")
        nix build '.#checks.x86_64-linux.hyprshell-clippy' -L
        ;;
    "fmt")
        cargo fmt -- --check
        ;;
    "fmt-nix")
        nix build '.#checks.x86_64-linux.hyprshell-fmt' -L
        ;;
    "test")
        cargo nextest run --cargo-profile dev --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib
        ;;
    "test-nix")
        nix build '.#checks.x86_64-linux.hyprshell-test' -L
        ;;
    "check-nix-configs")
        nix build '.#checks.x86_64-linux.hyprshell-check-nix-configs' -L
        ;;
    "check-features")
        nix build '.#checks.x86_64-linux.hyprshell-check-all-feature-combinations' -L
        ;;
    "all")
        cargo build --profile dev --locked && \
        cargo clippy --profile dev --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib -- --deny warnings && \
        cargo fmt -- --check && \
        cargo nextest run --cargo-profile dev --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib
        ;;
    "all-nix")
        nix build '.#checks.x86_64-linux.hyprshell-config-check' -L && \
        nix build '.#checks.x86_64-linux.hyprshell-clippy' -L && \
        nix build '.#checks.x86_64-linux.hyprshell-fmt' -L && \
        nix build '.#checks.x86_64-linux.hyprshell-test' -L
        ;;
    *)
        echo "Error: Unknown command '$1'"
        help
        ;;
esac
