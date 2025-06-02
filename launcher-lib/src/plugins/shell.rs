use crate::plugins::{Identifier, PluginNames, StaticLaunchOption};
use exec_lib::run::run_program;
use std::path::PathBuf;

pub fn get_static_options(matches: &mut Vec<StaticLaunchOption>) {
    matches.push(StaticLaunchOption {
        iden: Identifier {
            plugin: PluginNames::Shell,
            identifier: None,
        },
        key: 'r',
        text: Box::from("Shell"),
        details: Box::from("Run a command in a shell"),
        icon: Some(PathBuf::from("bash").into_boxed_path()),
    });
}

pub fn launch_option(text: &str, default_terminal: &Option<Box<str>>) -> bool {
    run_program(text, None, false, default_terminal);
    true
}

pub(crate) fn get_chars() -> Vec<char> {
    vec!['r']
}
