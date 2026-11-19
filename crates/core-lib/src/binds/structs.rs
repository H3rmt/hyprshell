#[derive(Debug)]
pub struct ExecBind {
    pub mods: Vec<&'static str>,
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
