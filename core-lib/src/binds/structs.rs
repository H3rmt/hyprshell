#[derive(Debug)]
pub struct ExecBind {
    pub mods: Vec<&'static str>,
    pub key: Box<str>,
    pub exec: Box<str>,
}
