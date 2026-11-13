#[derive(Debug)]
pub struct ExecBind {
    pub mods: Vec<&'static str>,
    pub key: Box<str>,
    pub exec: String,
    pub release: bool,
    pub repeating: bool,
    pub transparent: bool,
    pub desc: String,
}
