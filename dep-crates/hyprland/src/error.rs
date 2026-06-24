#[derive(Debug, derive_more::Display)]
/// Error that unifies different error types used by Hyprland-rs
pub enum HyprError {
    /// Error coming from serde
    SerdeError(serde_json::Error),
    /// Error coming from std::io
    IoError(io::Error),
    /// Error from failing to parse HyprColor
    InvalidHyprColorFormat,
    /// Error from failing to parse HyprGradient
    InvalidHyprGradiantFormat,
    /// Error that occurs when parsing UTF-8 string
    FromUtf8Error(std::string::FromUtf8Error),
    /// Dispatcher returned non `ok` value
    #[display("A dispatcher returned a non-`ok`, value which is probably an error: {_0}")]
    NotOkDispatch(String),
    /// Error when interacting with Hyprpaper.
    #[cfg(feature = "hyprpaper")]
    Hyprpaper(crate::hyprpaper::Error),
    /// Keyword does not exist
    InvalidOptionKey(String),
    /// Unparsable Option
    InvalidOptionValue,
    /// Internal Hyprland error
    Internal(String),
    /// Error that occurs for other reasons. Avoid using this.
    Other(String),
}
impl HyprError {
    /// Try to get an owned version of the internal error.
    ///
    /// Some dependencies of hyprland do not impl Clone in their error types. This is a partial workaround.
    ///
    /// If it succeeds, it returns the owned version of HyprError in Ok(). Otherwise, it returns a reference to the error type.
    pub fn try_as_cloned(&self) -> Result<Self, &Self> {
        match self {
            Self::SerdeError(_) => Err(self),
            Self::IoError(_) => Err(self),
            Self::FromUtf8Error(e) => Ok(Self::FromUtf8Error(e.clone())),
            Self::NotOkDispatch(s) => Ok(Self::NotOkDispatch(s.clone())),
            Self::InvalidHyprColorFormat => Ok(Self::InvalidHyprColorFormat),
            Self::InvalidHyprGradiantFormat => Ok(Self::InvalidHyprGradiantFormat),
            #[cfg(feature = "hyprpaper")]
            Self::Hyprpaper(_) => Err(self),
            Self::InvalidOptionKey(key) => Ok(Self::InvalidOptionKey(key.clone())),
            Self::InvalidOptionValue => Ok(Self::InvalidOptionValue),
            Self::Internal(s) => Ok(Self::Internal(s.clone())),
            Self::Other(s) => Ok(Self::Other(s.clone())),
        }
    }
    /// Create a Hyprland error with dynamic data.
    #[inline(always)]
    pub fn other<S: Into<String>>(other: S) -> Self {
        Self::Other(other.into())
    }
}

impl From<io::Error> for HyprError {
    fn from(error: io::Error) -> Self {
        HyprError::IoError(error)
    }
}

impl From<serde_json::Error> for HyprError {
    fn from(error: serde_json::Error) -> Self {
        HyprError::SerdeError(error)
    }
}

impl From<std::string::FromUtf8Error> for HyprError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        HyprError::FromUtf8Error(error)
    }
}

impl error::Error for HyprError {}

/// Internal macro to return a Hyprland error
macro_rules! hypr_err {
    ($fmt:literal) => {
        return Err($crate::error::HyprError::Internal(format!($fmt)))
    };
    (other $fmt:literal) => {
        return Err($crate::error::HyprError::Other(format!($fmt)))
    };
    ($fmt:literal $(, $value:expr)+) => {
        return Err($crate::error::HyprError::Internal(format!($fmt $(, $value)+)))
    };
    (other $fmt:literal $(, $value:expr)+) => {
        return Err($crate::error::HyprError::Other(format!($fmt $(, $value)+)))
    };
}

pub(crate) use hypr_err;
use std::{error, io};
