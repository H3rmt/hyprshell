use anyhow::bail;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Super,
    None,
}

#[allow(clippy::must_use_candidate)]
impl Modifier {
    pub fn to_l_key(&self) -> String {
        self.to_keysym_l().to_string()
    }
    pub const fn to_keysym_l(&self) -> &'static str {
        // XKB keysym names (xkbcommon-keysyms.h). xkbcommon's case-insensitive
        // lookup accepts e.g. "alt_l", but "ctrl_l" is NOT a valid XKB keysym
        // (the canonical name is "Control_L"). So we return canonical XKB names.
        match self {
            Self::Alt => "Alt_L",
            Self::Ctrl => "Control_L",
            Self::Super => "Super_L",
            Self::None => "",
        }
    }
    pub const fn to_keysym_r(&self) -> &'static str {
        match self {
            Self::Alt => "Alt_R",
            Self::Ctrl => "Control_R",
            Self::Super => "Super_R",
            Self::None => "",
        }
    }
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Alt => "alt",
            Self::Ctrl => "ctrl",
            Self::Super => "super",
            Self::None => "none",
        }
    }
}

impl<'de> Deserialize<'de> for Modifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ModVisitor;
        impl Visitor<'_> for ModVisitor {
            type Value = Modifier;
            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("one of: alt, ctrl, super, none")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .try_into()
                    .map_err(|_e| E::unknown_variant(value, &["alt", "ctrl", "super", "none"]))
            }
        }
        deserializer.deserialize_str(ModVisitor)
    }
}

impl Serialize for Modifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_str();
        serializer.serialize_str(s)
    }
}

impl TryFrom<&str> for Modifier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "alt" => Ok(Self::Alt),
            "ctrl" | "control" => Ok(Self::Ctrl),
            "super" | "win" | "windows" | "meta" => Ok(Self::Super),
            "none" | "" => Ok(Self::None),
            other => bail!("Invalid modifier: {other}"),
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alt => write!(f, "Alt"),
            Self::Ctrl => write!(f, "Ctrl"),
            Self::Super => write!(f, "Super"),
            Self::None => write!(f, "None"),
        }
    }
}
