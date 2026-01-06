use crate::plugins::{Identifier, PluginNames, PluginReturn, StaticLaunchOption};
use core_lib::WarnWithDetails;
use exec_lib::run::run_program;
use relm4::adw::gtk::gdk::Key;
use std::path::PathBuf;
use tracing::{debug, trace};

pub fn get_static_options(matches: &mut Vec<StaticLaunchOption>, default_terminal: Option<&str>) {
    matches.push(StaticLaunchOption {
        iden: Identifier::plugin(PluginNames::Terminal),
        key: 't',
        text: Box::from("Terminal"),
        details: Box::from("Run a command in a terminal"),
        icon: Some(
            PathBuf::from(default_terminal.map_or("system-run", |term| match term {
                // random fix for alacritty icon
                "alacritty" => "Alacritty",
                other => other,
            }))
            .into_boxed_path(),
        ),
    });
    trace!("Added static terminal option");
}

pub fn launch_option(text: &str, default_terminal: Option<&str>) -> PluginReturn {
    if text.is_empty() {
        debug!("No text to run in terminal");
        return PluginReturn {
            show_animation: false,
        };
    }
    run_program(
        // exec shell to prevent needing 2 exits
        // echo to make the shell look better and show the executed command
        &format!("$SHELL -c \"echo '> {text}';{text};echo;exec $SHELL\""),
        None,
        true,
        default_terminal,
    )
    .warn_details("Failed to run program");
    PluginReturn {
        show_animation: true,
    }
}

pub fn get_chars() -> Vec<Key> {
    vec![Key::t]
}
