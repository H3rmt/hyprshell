# hyprshell-config-lib

This crate owns hyprshell configuration parsing, writing, and migration.
It converts on-disk config formats into the internal structures the app uses at runtime.
It also handles versioned migration paths so existing user config can keep working as the schema evolves.

- Main repo: https://github.com/h3rmt/hyprshell
- Docs: https://docs.rs/hyprshell-config-lib
