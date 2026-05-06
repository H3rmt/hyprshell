//! # Data module
//!
//! This module provides functions for getting information on the compositor
//!
//! ## Usage
//!
//! here is a example of every function in use! (blocking)
//! ```rust
//! use hyprland::data::*;
//! use hyprland::prelude::*;
//! use hyprland::Result;
//!
//! fn main() -> Result<()> {
//!     let instance = &hyprland::instance::Instance::from_current_env()?;
//!
//!     let monitors = Monitors::instance_get(instance)?.to_vec();
//!     println!("{monitors:#?}");
//!
//!     let workspaces = Workspaces::instance_get(instance)?.to_vec();
//!     println!("{workspaces:#?}");
//!
//!     let clients = Clients::instance_get(instance)?.to_vec();
//!     println!("{clients:#?}");
//!
//!     let active_window = Client::instance_get_active(instance)?;
//!     println!("{active_window:#?}");
//!
//!     let layers = Layers::instance_get(instance)?;
//!     println!("{layers:#?}");
//!
//!     let devices = Devices::instance_get(instance)?;
//!     println!("{devices:#?}");
//!
//!     let version = Version::instance_get(instance)?;
//!     println!("{version:#?}");
//!
//!     let cursor_pos = CursorPosition::instance_get(instance)?;
//!     println!("{cursor_pos:#?}");
//!     Ok(())
//! }
//! ```

#[macro_use]
mod macros;

use crate::shared::*;

#[cfg(feature = "ahash")]
use ahash::HashMap;
#[cfg(not(feature = "ahash"))]
use std::collections::HashMap;

mod regular;

/// Helpers data commands, these use other hyprctl commands to create new ones!
mod helpers;

pub use crate::data::helpers::*;

pub use crate::data::regular::*;

//// This module provides async function calls
//pub mod asynchronous;

//// This module provides blocking function calls
//pub mod blocking;
