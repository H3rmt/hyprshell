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
