use crate::plugin::LaunchItem;
use config_lib::ActionsPluginAction;
use core_lib::transfer::{Identifier, PluginName};
use std::path::PathBuf;

impl From<ActionsPluginAction> for LaunchItem {
    #[allow(clippy::too_many_lines)]
    fn from(action: ActionsPluginAction) -> Self {
        match action {
            ActionsPluginAction::LockScreen => {
                let command: Box<str> = Box::from("loginctl lock-session");
                Self {
                    name: Box::from("Lock Screen"),
                    keywords: Box::new([Box::from("user")]),
                    details: command.clone(),
                    details_long: Some(Box::from("Lock the screen")),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-lock-screen").into_boxed_path()),
                    children: Box::new([]),
                }
            }
            ActionsPluginAction::Logout => {
                let command: Box<str> = Box::from("loginctl terminate-session self");
                Self {
                    name: Box::from("Log Out"),
                    keywords: Box::new([Box::from("user")]),
                    details: command.clone(),
                    details_long: Some(Box::from("Log out of the session")),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-log-out").into_boxed_path()),
                    children: Box::new([]),
                }
            }
            ActionsPluginAction::Hibernate => {
                let command: Box<str> = Box::from("systemctl hibernate");
                Self {
                    name: Box::from("Hibernate"),
                    keywords: Box::new([]),
                    details: command.clone(),
                    details_long: Some(Box::from(
                        "Writes RAM to disk, then powers off. Boots on wake",
                    )),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                    children: Box::new([Self {
                        name: Box::from("Hybrid Sleep"),
                        keywords: Box::new([]),
                        icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                        details: Box::from("systemctl hybrid-sleep"),
                        details_long: Some(Box::from(
                            "Writes RAM to disk, then enters low-power sleep. Enables fast wakeup time",
                        )),
                        bonus_score: 0,
                        takes_args: false,
                        iden: Identifier::data(
                            PluginName::Actions,
                            Box::from("systemctl hybrid-sleep"),
                        ),
                        children: Box::new([]),
                    }]),
                }
            }
            ActionsPluginAction::Reboot => {
                let command: Box<str> = Box::from("systemctl reboot");
                Self {
                    name: Box::from("Reboot / Restart"),
                    keywords: Box::new([]),
                    details: command.clone(),
                    details_long: Some(Box::from("Reboot the computer")),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-reboot").into_boxed_path()),
                    children: Box::new([]),
                }
            }
            ActionsPluginAction::Shutdown => {
                let command: Box<str> = Box::from("systemctl poweroff");
                Self {
                    name: Box::from("Shutdown / Poweroff"),
                    keywords: Box::new([]),
                    details: command.clone(),
                    details_long: Some(Box::from("Shut down the computer")),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-shutdown").into_boxed_path()),
                    children: Box::new([]),
                }
            }
            ActionsPluginAction::Suspend => {
                let command: Box<str> = Box::from("systemctl suspend");
                Self {
                    name: Box::from("Sleep / Suspend"),
                    keywords: Box::new([]),
                    details: command.clone(),
                    details_long: Some(Box::from("Enters low-power sleep")),
                    bonus_score: 0,
                    takes_args: false,
                    iden: Identifier::data(PluginName::Actions, command),
                    icon: Some(PathBuf::from("system-suspend").into_boxed_path()),
                    children: Box::new([Self {
                        name: Box::from("Suspend Then Hibernate"),
                        keywords: Box::new([]),
                        icon: Some(PathBuf::from("system-hibernate").into_boxed_path()),
                        details: Box::from("systemctl suspend-then-hibernate"),
                        details_long: Some(Box::from(
                            "Low-power sleep, then hibernate after some time",
                        )),
                        bonus_score: 0,
                        takes_args: false,
                        iden: Identifier::data(
                            PluginName::Actions,
                            Box::from("systemctl suspend-then-hibernate"),
                        ),
                        children: Box::new([]),
                    }]),
                }
            }
            ActionsPluginAction::Custom(c) => Self {
                name: c.names[0].clone(),
                keywords: Box::new([]),
                icon: Some(c.icon),
                details: c.details,
                details_long: Some(c.command.clone()),
                bonus_score: 0,
                takes_args: c.command.contains("{}"),
                iden: Identifier::data(PluginName::Actions, c.command),
                children: Box::new([]),
            },
        }
    }
}
