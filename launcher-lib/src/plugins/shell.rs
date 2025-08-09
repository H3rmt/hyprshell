use crate::plugins::{Identifier, PluginNames, StaticLaunchOption};
use core_lib::WarnWithDetails;
use exec_lib::run::run_program;
use gtk::gdk::Key;
use std::path::PathBuf;
use tracing::{debug, trace};

pub fn get_static_options(matches: &mut Vec<StaticLaunchOption>) {
    matches.push(StaticLaunchOption {
        iden: Identifier::plugin(PluginNames::Shell),
        key: 'r',
        text: Box::from("Shell"),
        details: Box::from("Run a command in a shell"),
        icon: Some(PathBuf::from("bash").into_boxed_path()),
    });
    trace!("Added static shell option");
}

pub fn launch_option(text: &str, default_terminal: Option<&str>) -> bool {
    if text.is_empty() {
        debug!("No text to run in shell");
        return false;
    }
    run_program(text, None, false, default_terminal).warn_details("Failed to run program");
    true
}

pub fn get_chars() -> Vec<Key> {
    vec![Key::r]
}
