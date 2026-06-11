use crate::plugin::{PluginItem, PluginReturn};
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use exec_lib::run::run_program;
use relm4::adw::gtk::gdk::Key;
use std::path::PathBuf;
use tracing::debug;

pub fn get_static_items() -> Vec<PluginItem> {
    vec![PluginItem {
        iden: Identifier::plugin(PluginName::Shell),
        key: 'r',
        text: Box::from("Shell"),
        details: Box::from("Run a command in a shell"),
        icon: Some(PathBuf::from("bash").into_boxed_path()),
    }]
}

pub fn launch_option(text: &str, default_terminal: Option<&str>) -> PluginReturn {
    if text.is_empty() {
        debug!("No text to run in shell");
        return PluginReturn {
            show_animation: false,
        };
    }
    run_program(text, None, false, default_terminal).warn_details("Failed to run program");
    PluginReturn {
        show_animation: true,
    }
}

pub fn get_chars() -> Vec<Key> {
    vec![Key::r]
}
