#!/usr/bin/env bash
set -euxo pipefail

# Define the libs
declare -a libs=("hyprshell-config-lib" "hyprshell-core-lib" "hyprshell-exec-lib" "hyprshell-launcher-lib" "hyprshell-windows-lib" "hyprshell-hyprland-plugin" "hyprshell-clipboard-lib" "hyprshell-config-edit-lib")

for lib in "${libs[@]}"; do
    echo "Fixing $lib..."
    cargo fix --lib -p "$lib" --allow-dirty
done

echo "Fixing main binary"
cargo fix -p hyprshell --all --allow-dirty

echo "Building main binary"
cargo clippy --all-targets --all-features -- -D warnings