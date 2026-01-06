use crate::plugins::{Identifier, PluginNames, PluginReturn, SortableLaunchOption};
use config_lib::actions::ToAction;
use config_lib::{ActionsPluginActionCustom, ActionsPluginConfig};
use core_lib::WarnWithDetails;
use tracing::{info, trace};

pub fn get_actions_options(
    matches: &mut Vec<SortableLaunchOption>,
    text: &str,
    config: &ActionsPluginConfig,
) {
    if text.is_empty() {
        return;
    }
    let lower_text = text.to_ascii_lowercase();

    let actions = config
        .actions
        .iter()
        .cloned()
        .map(ToAction::to_action)
        .collect::<Vec<ActionsPluginActionCustom>>();

    for action in actions {
        if action.names.iter().any(|name| {
            // searching ki should show kill, searching kill kitty should show kill
            name.to_ascii_lowercase().starts_with(&lower_text)
                || lower_text.starts_with(&name.to_ascii_lowercase())
        }) || text.eq("actions")
        {
            let name = action
                .names
                .iter()
                .find(|name| {
                    name.to_ascii_lowercase().starts_with(&lower_text)
                        || lower_text.starts_with(&name.to_ascii_lowercase())
                })
                .expect("cant happen we already searched");
            let name_lower = name.to_ascii_lowercase();
            let mut command = action.command.clone();
            let mut grayed = false;
            if command.contains("{}") {
                trace!(
                    "Action command contains '{{}}', replacing <{text}> with stripped ({name_lower}) text"
                );
                let stripped_text = {
                    let trimmed = text.trim_start_matches(&*name_lower).trim();
                    if trimmed.len() == text.len() {
                        ""
                    } else {
                        trimmed
                    }
                };
                if stripped_text.is_empty() {
                    grayed = true;
                }
                command = Box::from(command.replace("{}", stripped_text));
            }
            trace!("Added action option: {}", command);
            matches.push(SortableLaunchOption {
                icon: Some(action.icon),
                name: name.clone(),
                details: action.details,
                details_long: Some(command.clone()),
                score: 30,
                grayed,
                iden: Identifier::data(PluginNames::Actions, command),
                details_menu: vec![],
            });
        }
    }
}

pub fn run_action(data: Option<&str>) -> PluginReturn {
    if let Some(data) = data {
        if cfg!(debug_assertions) && std::env::var("HYPRSHELL_RUN_ACTIONS_IN_DEBUG").is_err() {
            info!("Not running action: {} (debug mode)", data);
        } else {
            info!("Running action: {}", data);
            exec_lib::run::run_program(data, None, false, None)
                .warn_details("Failed to run command");
        }
    }

    PluginReturn {
        show_animation: true,
    }
}
