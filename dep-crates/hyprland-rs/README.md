# Hyprland-rs

[![Crates.io](https://img.shields.io/crates/v/hyprland)](https://crates.io/crates/hyprland)
![Crates.io](https://img.shields.io/crates/d/hyprland)
[![Crates.io](https://img.shields.io/crates/l/hyprland)](https://www.gnu.org/licenses/gpl-3.0.html)
[![docs.rs](https://img.shields.io/docsrs/hyprland)](https://docs.rs/hyprland)
[![Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue)](https://github.com/hyprwm/Hyprland)
[![Discord](https://img.shields.io/discord/1055990214411169892?label=discord)](https://discord.gg/zzWqvcKRMy)

An unofficial rust wrapper for Hyprland's IPC

## Help Wanted!
We need help with developing the next version of hyprland-rs `0.4`,
if you know how to do the things in <https://github.com/hyprland-community/hyprland-rs/milestone/4>
contributions in those areas would be greatly appreciated!

## Disclaimer

If something doesn't work, doesn't matter what,
make sure you are on the latest version (or commit) of Hyprland before making an issue!

## Getting started!

Let's get started with Hyprland-rs!

### Adding to your project

Add the code below to the dependencies section of your Cargo.toml file!

```toml
hyprland = "0.4.0-beta.3"
```

### Reading the docs

Hyprland-rs has a ton of types (and some really long ones)! Its important you know how the ones you will be using work!
The docs can be found at [docs.rs/hyprland](https://docs.rs/hyprland)

#### Master version

If Hyprland-rs is broken (or other reason) and is taking too long for a release to come out,
you can use the master branch in Cargo (will not allow the crate to be published to `crates.io`):

```toml
hyprland = { git = "https://github.com/hyprland-community/hyprland-rs", branch = "master" }
```

### What this crate provides

This crate provides 6 modules (+1 for shared things)

- `data` for getting information on the compositor
- `event_listener` which provides the `EventListener` struct for listening for events
- `dispatch` for calling dispatchers
- `keyword` for dealing with config option (aka keywords)
- `config::binds` for changing binds (in future `config` might have config generation)
- `ctl` for calling hyprctl commands

## Example Usage

Check the examples in the [`examples` directory](https://github.com/hyprland-community/hyprland-rs/tree/master/examples)
