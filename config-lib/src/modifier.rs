use anyhow::bail;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Super,
    Shift,
}

impl Modifier {
    pub fn to_l_key(&self) -> String {
        match self {
            Self::Alt => "alt_l".to_string(),
            Self::Ctrl => "ctrl_l".to_string(),
            Self::Super => "super_l".to_string(),
            Self::Shift => "shift_l".to_string(),
        }
    }
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Alt => "alt",
            Self::Ctrl => "ctrl",
            Self::Super => "super",
            Self::Shift => "shift",
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
                fmt.write_str("one of: alt, ctrl, super, shift")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .try_into()
                    .map_err(|_e| E::unknown_variant(value, &["alt", "ctrl", "super", "shift"]))
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
            "Alt" | "alt" => Ok(Self::Alt),
            "Ctrl" | "ctrl" | "control" | "Control" => Ok(Self::Ctrl),
            "Super" | "super" | "Win" | "win" | "windows" | "Windows" => Ok(Self::Super),
            "Shift" | "shift" => Ok(Self::Shift),
            other => bail!("Invalid modifier: {}", other),
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alt => write!(f, "Alt"),
            Self::Ctrl => write!(f, "Ctrl"),
            Self::Super => write!(f, "Super"),
            Self::Shift => write!(f, "Shift"),
        }
    }
}
