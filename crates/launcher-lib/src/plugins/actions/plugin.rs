use crate::plugin::{LaunchItem, PluginReturn};
use config_lib::{ActionsPluginActionCustom, ActionsPluginConfig};
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use std::path::PathBuf;
use tracing::{error, info, trace};

pub fn get_launch_items(config: &ActionsPluginConfig) -> Vec<LaunchItem> {
    config
        .actions
        .iter()
        .cloned()
        .map(LaunchItem::from)
        .collect::<Vec<LaunchItem>>()
}

pub fn run_action(data: Option<&str>, text: &str, data_additional: Option<&str>) -> PluginReturn {
    if let Some(command) = data {
        let mut command = command.to_string();
        if command.contains("{}") {
            if let Some(action_name) = data_additional {
                let stripped_text = text[action_name.len()..].trim();
                trace!(
                    "Action command contains '{{}}', replacing {{}} in <{command}> with stripped ({stripped_text}) text extracted from <{text}>"
                );
                command = command.replace("{}", stripped_text);
            } else {
                error!("Action command contains '{{}}', but no additional data was provided");
                return PluginReturn {
                    show_animation: false,
                };
            }
        }

        if cfg!(debug_assertions) && std::env::var("HYPRSHELL_RUN_ACTIONS_IN_DEBUG").is_err() {
            info!("Not running action: {command} (debug mode)");
        } else {
            info!("Running action: {command}");
            exec_lib::run::run_program(&command, None, false, None)
                .warn_details("Failed to run command");
        }
    }

    PluginReturn {
        show_animation: true,
    }
}
