//! # The Shared Module
//!
//! This module provides shared private and public functions, structs, enum, and types
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{env, fmt};

/// The address struct holds a address as a tuple with a single value
/// and has methods to reveal the address in different data formats
#[derive(
    Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, derive_more::Display,
)]
pub struct Address(String);
impl Address {
    #[inline(always)]
    pub(crate) fn fmt_new(address: &str) -> Self {
        // this way is faster than std::fmt
        Self("0x".to_owned() + address)
    }
    /// This creates a new address from a value that implements [ToString]
    pub fn new<T: ToString>(string: T) -> Self {
        let str = string.to_string();
        if str.starts_with("0x") {
            Self(str)
        } else {
            Self("0x".to_owned() + str.as_str())
        }
    }
}

/// This trait provides a standardized way to get data
pub trait HyprData {
    /// This method gets the data
    fn get() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data
    fn instance_get(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
}

/// This trait provides a standardized way to get data in a from of a vector
pub trait HyprDataVec<T>: HyprData {
    /// This method returns a vector of data
    fn to_vec(self) -> Vec<T>;
}

/// Trait for helper functions to get the active of the implementor
pub trait HyprDataActive {
    /// This method gets the active data
    fn get_active() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data
    fn instance_get_active(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
}

/// Trait for helper functions to get the active of the implementor, but for optional ones
pub trait HyprDataActiveOptional {
    /// This method gets the active data
    fn get_active() -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data
    fn instance_get_active(instance: &Instance) -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Option<Self>>
    where
        Self: Sized;
}

/// This type provides the id used to identify workspaces
/// > its a type because it might change at some point
pub type WorkspaceId = i32;

/// This type provides the id used to identify monitors
/// > its a type because it might change at some point
pub type MonitorId = i128;

#[inline]
fn ser_spec_opt(opt: &Option<String>) -> String {
    match opt {
        Some(name) => "special:".to_owned() + name,
        None => "special".to_owned(),
    }
}

/// This enum holds workspace data
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Display, PartialOrd, Ord)]
#[serde(untagged)]
pub enum WorkspaceType {
    /// A named workspace
    Regular(
        /// The name
        String,
    ),
    /// The special workspace
    #[display("{}", ser_spec_opt(_0))]
    Special(
        /// The name, if exists
        Option<String>,
    ),
}

impl From<&WorkspaceType> for String {
    fn from(value: &WorkspaceType) -> Self {
        value.to_string()
    }
}
macro_rules! from {
    ($($ty:ty),+$(,)?) => {
        $(
            impl TryFrom<$ty> for WorkspaceType {
                type Error = crate::error::HyprError;
                fn try_from(int: $ty) -> Result<Self, Self::Error> {
                    match int {
                        1.. => Ok(WorkspaceType::Regular(int.to_string())),
                        _ => crate::error::hypr_err!("Conversion error: Unrecognised id"),
                    }
                }
            }
        )+
    };
}
from![u8, u16, u32, u64, usize, i8, i16, i32, i64, isize];

impl Hash for WorkspaceType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            WorkspaceType::Regular(name) => name.hash(state),
            WorkspaceType::Special(value) => match value {
                Some(name) => name.hash(state),
                None => "".hash(state),
            },
        }
    }
}

pub(crate) fn get_hypr_path() -> crate::Result<PathBuf> {
    let mut buf = if let Some(runtime_path) = env::var_os("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(runtime_path)
    } else if let Ok(uid) = env::var("UID") {
        std::path::PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        hypr_err!("Could not find XDG_RUNTIME_DIR or UID");
    };
    buf.push("hypr");
    Ok(buf)
}

/// This enum defines the possible command flags that can be used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandFlag {
    /// The JSON flag.
    #[default]
    JSON,
    /// An empty flag.
    Empty,
}

/// This struct defines the content of a command, which consists of a flag and a data string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandContent {
    /// The flag for the command.
    pub flag: CommandFlag,
    /// The data string for the command.
    pub data: String,
}

impl CommandContent {
    /// Converts the command content to a byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyprland::shared::*;
    ///
    /// let content = CommandContent { flag: CommandFlag::JSON, data: "foo".to_string() };
    /// let bytes = content.as_bytes();
    /// assert_eq!(bytes, b"j/foo");
    /// ```
    pub fn as_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl fmt::Display for CommandContent {
    /// Formats the command content as a string for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyprland::shared::*;
    ///
    /// let content = CommandContent { flag: CommandFlag::JSON, data: "foo".to_string() };
    /// let s = format!("{}", content);
    /// assert_eq!(s, "j/foo");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.flag {
            CommandFlag::JSON => write!(f, "j/{}", &self.data),
            CommandFlag::Empty => write!(f, "/{}", &self.data),
        }
    }
}

/// Creates a `CommandContent` instance with the given flag and formatted data.
///
/// # Arguments
///
/// * `$flag` - A `CommandFlag` variant (`JSON` or `Empty`) that represents the flag for the command.
/// * `$($k:tt)*` - A format string and its arguments to be used as the data in the `CommandContent` instance.
#[macro_export]
macro_rules! command {
    ($flag:ident, $($k:tt)*) => {{
        $crate::shared::CommandContent {
            flag: $crate::shared::CommandFlag::$flag,
            data: format!($($k)*),
        }
    }};
}
use crate::error::hypr_err;
use crate::instance::Instance;
pub use command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[allow(missing_docs)]
/// Enum for mod keys used in bind combinations
pub enum Mod {
    #[display("SUPER")]
    SUPER,
    #[display("SHIFT")]
    SHIFT,
    #[display("ALT")]
    ALT,
    #[display("CTRL")]
    CTRL,
    #[display("")]
    NONE,
}
