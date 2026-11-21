#[derive(Debug)]
pub struct ExecBind {
    pub mods: Vec<Mod>,
    pub key: Box<str>,
    pub exec: String,
    pub behavior: ExecBindBehavior,
    pub desc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecBindBehavior {
    /// Trigger on key press without repeating while the key remains held.
    PressOnce,
    Repeating,
    Release,
    TransparentReleaseIgnoringMods {
        /// Only run if neither of these keysyms is still held.
        unless_keys_down: Option<(&'static str, &'static str)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mod {
    Alt,
    Super,
    Ctrl,
    Shift,
}

impl From<crate::Modifier> for Mod {
    fn from(value: crate::Modifier) -> Self {
        match value {
            crate::Modifier::Alt => Self::Alt,
            crate::Modifier::Ctrl => Self::Ctrl,
            crate::Modifier::Super => Self::Super,
        }
    }
}

pub fn is_modifier_keysym(key: &str) -> bool {
    // We only check for the 3 mod types we actually support (Alt, Ctrl, Super)
    const MODIFIER_KEYSYMS: [&str; 10] = [
        "Super_L",
        "Super_R",
        "Alt_L",
        "Alt_R",
        "Control_L",
        "Control_R",
        "Meta_L",
        "Meta_R",
        "Hyper_L",
        "Hyper_R",
    ];

    MODIFIER_KEYSYMS
        .iter()
        .any(|keysym| key.eq_ignore_ascii_case(keysym))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_modifier_keysym_true() {
        for keysym in [
            "sUpEr_l",
            "Super_R",
            "aLt_l",
            "Alt_R",
            "cOnTrOl_l",
            "Control_R",
            // "sHiFt_l",
            // "Shift_R",
            "mEtA_l",
            "Meta_R",
            "hYpEr_l",
            "Hyper_R",
        ] {
            assert!(is_modifier_keysym(keysym), "{keysym} invalid");
        }
    }

    #[test]
    fn is_modifier_keysym_false() {
        for keysym in ["Tab", "Shift", "Shift_R", "Shift_L", "grave", "a"] {
            assert!(!is_modifier_keysym(keysym), "{keysym} invalid");
        }
    }
}
