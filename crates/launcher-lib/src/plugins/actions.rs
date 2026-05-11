use crate::plugins::{PluginReturn, SortableLaunchOption};
use config_lib::actions::ToAction;
use config_lib::{ActionsPluginActionCustom, ActionsPluginConfig};
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use tracing::{error, info, trace};

pub fn get_actions_options(matches: &mut Vec<SortableLaunchOption>, config: &ActionsPluginConfig) {
    let actions = config
        .actions
        .iter()
        .cloned()
        .map(ToAction::to_action)
        .collect::<Vec<ActionsPluginActionCustom>>();

    for action in actions {
        let takes_args = action.command.contains("{}");
        trace!("Added action option: {}", action.command);
        if takes_args {
            // we need to create actions with a single name, because the name needs to be removed later
            for name in action.names.iter() {
                matches.push(SortableLaunchOption {
                    icon: Some(action.icon.clone()),
                    names: Box::from([name.clone()]),
                    details: action.details.clone(),
                    details_long: Some(action.command.clone()),
                    bonus_score: 0,
                    takes_args: true,
                    enabled: true,
                    iden: Identifier::data_additional(
                        PluginName::Actions,
                        action.command.clone(),
                        name.clone(),
                    ),
                    subactions: vec![],
                })
            }
        } else {
            matches.push(SortableLaunchOption {
                icon: Some(action.icon),
                names: Box::from(action.names),
                details: action.details,
                details_long: Some(action.command.clone()),
                bonus_score: 0,
                takes_args: false,
                enabled: true,
                iden: Identifier::data(PluginName::Actions, action.command),
                subactions: vec![],
            });
        }
    }
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
