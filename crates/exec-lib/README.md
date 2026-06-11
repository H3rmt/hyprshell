# hyprshell-exec-lib

This crate provides the low-level command execution helpers used throughout hyprshell.
It implements the Hyprland specific logic for loading data like windows and workspaces.

To make hyprshell work on different window managers you would in theory only have to reimplement
the public functions defined in this crate

- Main repo: https://github.com/h3rmt/hyprshell
- Docs: https://docs.rs/hyprshell-exec-lib
