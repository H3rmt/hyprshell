use crate::plugin::{LaunchItem, PluginReturn};
use config_lib::ActionsPluginConfig;
use core_lib::WarnWithDetails;
use tracing::{error, info, trace};

pub fn get_launch_items(config: &ActionsPluginConfig) -> Vec<LaunchItem> {
    config
        .actions
        .iter()
        .cloned()
        .map(LaunchItem::from)
        .collect::<Vec<LaunchItem>>()
}

pub fn run_action(
    data: Option<&str>,
    text: &str,
    _data_additional: Option<&str>,
    args: Option<&str>,
) -> PluginReturn {
    if let Some(command) = data {
        let mut command = command.to_string();
        if command.contains("{}") {
            if let Some(args) = args {
                if args.trim().is_empty() {
                    error!("Action command contains '{{}}', but no arguments were provided");
                    return PluginReturn {
                        show_animation: false,
                    };
                }
                trace!(
                    "Action command contains '{{}}', replacing {{}} in <{command}> with extracted args ({args}) from <{text}>"
                );
                command = command.replace("{}", args.trim());
            } else {
                error!("Action command contains '{{}}', but no arguments were provided");
                return PluginReturn {
                    show_animation: false,
                };
            }
        }

        if cfg!(debug_assertions) && std::env::var("HYPRSHELL_RUN_ACTIONS_IN_DEBUG").is_err() {
            info!("Not running action: {command} (debug mode)");
        } else {
            info!("Running action: {command}");
            exec_lib::run::run_program(&command, None, false, None, true)
                .warn_details("Failed to run command");
        }
    }

    PluginReturn {
        show_animation: true,
    }
}
