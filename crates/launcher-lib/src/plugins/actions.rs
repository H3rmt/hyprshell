use crate::plugins::{LaunchChildItem, LaunchItem, PluginReturn};
use config_lib::actions::ToAction;
use config_lib::{ActionsPluginActionCustom, ActionsPluginConfig};
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use std::path::PathBuf;
use tracing::{error, info, trace};

fn hibernate_children() -> Box<[LaunchChildItem]> {
    Box::from([
        LaunchChildItem {
            name: Box::from("Hybrid Sleep"),
            icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
            details: Box::from("Suspend to RAM and disk"),
            details_long: Some(Box::from("systemctl hybrid-sleep")),
            enabled: true,
            iden: Identifier::data(PluginName::Actions, Box::from("systemctl hybrid-sleep")),
        },
        LaunchChildItem {
            name: Box::from("Suspend Then Hibernate"),
            icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
            details: Box::from("Suspend first, then hibernate later"),
            details_long: Some(Box::from("systemctl suspend-then-hibernate")),
            enabled: true,
            iden: Identifier::data(
                PluginName::Actions,
                Box::from("systemctl suspend-then-hibernate"),
            ),
        },
    ])
}

pub fn get_launch_items(matches: &mut Vec<LaunchItem>, config: &ActionsPluginConfig) {
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
                matches.push(LaunchItem {
                    icon: Some(action.icon.clone()),
                    names: Box::from([name.clone()]),
                    keywords: Box::from([]),
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
                    children: Box::from([]),
                })
            }
        } else {
            let children = if action.command.as_ref() == "systemctl hibernate" {
                hibernate_children()
            } else {
                Box::from([])
            };
            matches.push(LaunchItem {
                icon: Some(action.icon),
                names: Box::from(action.names),
                keywords: Box::from([]),
                details: action.details,
                details_long: Some(action.command.clone()),
                bonus_score: 0,
                takes_args: false,
                enabled: true,
                iden: Identifier::data(PluginName::Actions, action.command),
                children,
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
