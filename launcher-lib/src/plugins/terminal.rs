use crate::plugins::{Identifier, PluginNames, StaticLaunchOption};
use exec_lib::run::run_program;
use std::path::PathBuf;

pub fn get_static_options(
    matches: &mut Vec<StaticLaunchOption>,
    default_terminal: &Option<Box<str>>,
) {
    matches.push(StaticLaunchOption {
        iden: Identifier {
            plugin: PluginNames::Terminal,
            identifier: None,
        },
        key: 't',
        text: Box::from("Terminal"),
        details: Box::from("Run a command in a terminal"),
        icon: Some(
            PathBuf::from(
                default_terminal
                    .as_deref()
                    .map(|term| match term {
                        // random fix for alacritty icon
                        "alacritty" => "Alacritty",
                        other => other,
                    })
                    .unwrap_or("system-run"),
            )
            .into_boxed_path(),
        ),
    });
}

pub fn launch_option(text: &str, default_terminal: &Option<Box<str>>) -> bool {
    run_program(
        // exec shell to prevent needing 2 exits
        // echo to make the shell look better and show the executed command
        &format!("$SHELL -c \"echo '> {text}';{text};echo;exec $SHELL\""),
        None,
        true,
        default_terminal,
    );
    true
}

pub(crate) fn get_chars() -> Vec<char> {
    vec!['t']
}
