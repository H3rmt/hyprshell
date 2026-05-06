#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(async_fn_in_trait)]
#![cfg_attr(feature = "unsafe-impl", allow(unsafe_code))]
#![cfg_attr(not(feature = "unsafe-impl"), forbid(unsafe_code))]

#[macro_use]
extern crate hyprland_macros;
#[macro_use]
extern crate paste;

use crate::error::HyprError;
use crate::instance::Instance;
use std::sync::OnceLock;

/// This module provides several impls that are unsafe, for FFI purposes. Only use if you know what you are doing.
#[cfg(feature = "unsafe-impl")]
pub mod unsafe_impl;

/// This module provides shared things throughout the crate
pub mod shared;

/// This module provides functions for getting information on the compositor
#[cfg(feature = "data")]
pub mod data;

/// This module provides the EventListener struct for listening and acting upon for events
#[cfg(feature = "listener")]
pub mod event_listener;

/// This module is for calling dispatchers and changing keywords
#[cfg(feature = "dispatch")]
pub mod dispatch;

/// This module is for calling hyprctl **commands**, for getting data use [data]
#[cfg(feature = "ctl")]
pub mod ctl;

/// This module provides the stuff needed to mutate, and read Hyprland config values
#[cfg(feature = "keyword")]
pub mod keyword;

/// This module provides helpers to easily config Hyprland
#[cfg(feature = "config")]
pub mod config;

/// Holds the error type used throughout the crate
pub mod error;
/// Used to generate the Instances to interface with Hyprland
pub mod instance;

/// This module is for interacting with [hyprpaper] using its IPC feature
///
/// [hyprpaper]: https://wiki.hyprland.org/Hypr-Ecosystem/hyprpaper/
#[cfg(feature = "hyprpaper")]
pub mod hyprpaper;

/// The prelude module, this is to import all traits
pub mod prelude {
    pub use crate::shared::{HyprData, HyprDataActive, HyprDataActiveOptional, HyprDataVec};
    pub use hyprland_macros::async_closure;
}

mod async_import {
    #[cfg(all(feature = "async-lite", not(feature = "tokio")))]
    pub use async_net::unix::UnixStream;
    #[cfg(all(feature = "async-lite", not(feature = "tokio")))]
    pub use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
    #[cfg(feature = "tokio")]
    pub use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::UnixStream};
}

/// This type provides the result type used everywhere in Hyprland-rs
pub type Result<T> = std::result::Result<T, HyprError>;

static DEFAULT_INSTANCE: OnceLock<Instance> = OnceLock::new();

/// Returns the result of the DEFAULT_INSTANCE OnceLock
pub fn default_instance() -> std::result::Result<&'static Instance, HyprError> {
    if let Some(i) = DEFAULT_INSTANCE.get() {
        return Ok(i);
    }
    let instance = Instance::from_current_env()?;
    let _ = DEFAULT_INSTANCE.set(instance);
    #[allow(clippy::unwrap_used)] // We just set the instance, so it can never fail
    Ok(DEFAULT_INSTANCE.get().unwrap())
}

/// Returns the result of the DEFAULT_INSTANCE OnceLock
pub fn default_instance_panic() -> &'static Instance {
    #[allow(clippy::expect_used)]
    default_instance().expect(
        "Default instance could not get initialized, use `Instance::from_instance()` instead.",
    )
}
