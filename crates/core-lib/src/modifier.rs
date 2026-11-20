use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Super,
}

#[allow(clippy::must_use_candidate)]
impl Modifier {
    pub const fn to_keysym_l(&self) -> &'static str {
        // XKB keysym names (xkbcommon-keysyms.h). xkbcommon's case-insensitive
        // lookup accepts e.g. "alt_l", but "ctrl_l" is NOT a valid XKB keysym
        // (the canonical name is "Control_L"). So we return canonical XKB names.
        match self {
            Self::Alt => "Alt_L",
            Self::Ctrl => "Control_L",
            Self::Super => "Super_L",
        }
    }
    pub const fn to_keysym_r(&self) -> &'static str {
        match self {
            Self::Alt => "Alt_R",
            Self::Ctrl => "Control_R",
            Self::Super => "Super_R",
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alt => write!(f, "Alt"),
            Self::Ctrl => write!(f, "Ctrl"),
            Self::Super => write!(f, "Super"),
        }
    }
}
