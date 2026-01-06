# Development Guide

Welcome to the Hyprshell development guide. This document provides information on how to set up your environment, the project structure, and common development tasks.

## Prerequisites

To develop for Hyprshell, you need to have the following installed:

- **Rust**: Latest stable version (minimum `1.92.0`).
- **GTK4 & Libadwaita**: Development headers for GTK4 and Libadwaita.
- **GTK4 Layer Shell**: Development headers for [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell).
- **Hyprland**: Minimum version `0.52.1`. Development headers (`hyprland-devel`) are needed for the plugin.
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
    - `core-lib`: Fundamental types and utilities.
    - `config-lib`: Configuration loading, generation, and migration.
    - `config-edit-lib`: The GUI settings editor.
    - `exec-lib`: Hyprland specific logic and plugin management.
    - `launcher-lib`: Logic for the application launcher.
    - `windows-lib`: Logic for the window switcher.
    - `clipboard-lib`: Clipboard management and history.
    - `hyprland-plugin`: A C++ Hyprland plugin used to capture keyboard events.
- `dep-crates/`: Contains forks or local versions of external dependencies.
    - `hyprland-rs`: A fork of the Hyprland IPC library.
    - `wl-clipboard-rs`: A fork of the Wayland clipboard library.
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