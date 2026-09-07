# Development Guide

Welcome to the Hyprshell development guide. This document provides information on how to set up your environment, the project structure, and common development tasks.

## Prerequisites

To develop for Hyprshell, you need to have the following installed:

- **Rust**: Latest stable version (minimum `1.92.0`).
- **GTK4 & Libadwaita**: Development headers for GTK4 and Libadwaita.
- **GTK4 Layer Shell**: Development headers for [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell).
- **Hyprland**: Minimum version `0.55.0`. Development headers (`hyprland-devel`) are needed for the plugin.
- **just**: A handy command runner used for various development tasks.

### Installing `just`

We use [just](https://github.com/casey/just) to automate common tasks. You can install it using your package manager:

- **Arch Linux**: `sudo pacman -S just`
- **Fedora**: `sudo dnf install just`
- **Nix**: `nix-shell -p just` or add it to your flake.
- **Cargo**: `cargo install just`

## Project Structure

Hyprshell is organized as a Rust workspace with multiple crates and some vendored dependencies.

### Directories

- `crates/`: Contains the internal libraries that make up Hyprshell.
    - `clipboard-lib`: Clipboard management and history.
    - `core-lib`: Fundamental types and utilities.
    - `config-lib`: Configuration loading, generation, and migration.
    - `config-edit-lib`: The GUI settings editor.
    - `exec-lib`: Hyprland specific logic and plugin management.
    - `launcher-lib`: Logic for the application launcher.
    - `windows-lib`: Logic for the window switcher.
- `dep-crates/`: Contains forks or local versions of external dependencies.
    - `hyprland-rs`: A fork of the Hyprland IPC library.
    - `wl-clipboard-rs`: A fork of the Wayland clipboard library.
    - `relm4` / `relm4-components`: A fork of the Relm4 library.
- `src/`: Contains the main entry point for the `hyprshell` binary.
- `scripts/`: Various helper scripts for CI and development.
- `nix/`: Nix-related files for building and development shells.
- `docs/`: Documentation files.
- `packaging/`: Files for packaging Hyprshell.

## Common Tasks

We use `just` to run common development tasks. Run `just` without arguments to see a full list of available commands.

### Development

- `just run`: Run the application in debug mode (prints available commands).
- `just run release`: Run the application in release mode.
- `just run-run`: Run the application process in debug mode.

### Environment Variables

Useful environment variables for development:

- `HYPRSHELL_EXPERIMENTAL=1`: Enables experimental features.
- `HYPRSHELL_LOG_MODULE_PATH=1`: Adds module path to logs (use with `-vv`).

## Checking changes

Run `just check lint test` before submitting a change. The `cargo xtask` alias
runs the same check commands used by CI. Clippy checks all targets of the
application and maintained crates, including xtask; vendored dependencies retain
their own lint policies. These Rust checks also perform type checking.

CI also compiles and checks vendored tests and examples with:

```sh
cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings
```

This needs OpenSSL development headers for the vendored example dependencies.

For changes involving optional features, run:

```sh
bash scripts/check-all-feature-combinations.sh
```

This checks default and slim builds, then every combination of the independent
features in `Cargo.toml`. It checks the maintained libraries explicitly and treats
warnings as errors, so disabled-feature warnings cannot hide in dependencies.

Additional checks for the files they cover:

```sh
# Shell scripts and GitHub Actions, using ShellCheck and actionlint.
shellcheck scripts/*.sh scripts/ci/*.sh
actionlint

# Nix formatting, static checks, and evaluation of both supported architectures.
nixfmt --check flake.nix nix/*.nix
statix check flake.nix
statix check nix
deadnix --fail flake.nix nix
nix flake check --all-systems --no-build --no-write-lock-file
```

The Nix evaluation command does not build packages or activate a configuration.
