use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workarrounds {
    pub class_to_icon: Vec<ClassToIcon>,
}

impl Default for Workarrounds {
    fn default() -> Self {
        crate::workarrounds::io::structs::Workarrounds::default()
            .try_into()
            .expect("the default config invalid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassToIcon {
    pub class: Box<str>,
    pub icon: Box<Path>,
}
