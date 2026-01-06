use crate::{ActionsPluginAction, ActionsPluginActionCustom};
use std::path::PathBuf;

pub trait ToAction {
    fn to_action(self) -> ActionsPluginActionCustom;
}

impl ToAction for ActionsPluginAction {
    fn to_action(self) -> ActionsPluginActionCustom {
        match self {
            Self::LockScreen => ActionsPluginActionCustom {
                names: vec![Box::from("Lock Screen")],
                details: Box::from("Lock the screen"),
                command: Box::from("loginctl lock-session"),
                icon: PathBuf::from("system-lock-screen").into_boxed_path(),
            },
            Self::Hibernate => ActionsPluginActionCustom {
                names: vec![Box::from("Hibernate")],
                details: Box::from("Hibernate the computer"),
                command: Box::from("systemctl hibernate"),
                icon: PathBuf::from("system-hibernate").into_boxed_path(),
            },
            Self::Logout => ActionsPluginActionCustom {
                names: vec![Box::from("Log Out"), Box::from("Logout")],
                details: Box::from("Log out of the session"),
                command: Box::from("loginctl terminate-session self"),
                icon: PathBuf::from("system-log-out").into_boxed_path(),
            },
            Self::Reboot => ActionsPluginActionCustom {
                names: vec![Box::from("Reboot"), Box::from("Restart")],
                details: Box::from("Reboot the computer"),
                command: Box::from("systemctl reboot"),
                icon: PathBuf::from("system-reboot").into_boxed_path(),
            },
            Self::Shutdown => ActionsPluginActionCustom {
                names: vec![Box::from("Shut Down"), Box::from("Power off")],
                details: Box::from("Shut down the computer"),
                command: Box::from("systemctl poweroff"),
                icon: PathBuf::from("system-shutdown").into_boxed_path(),
            },
            Self::Suspend => ActionsPluginActionCustom {
                names: vec![Box::from("Sleep"), Box::from("Suspend")],
                details: Box::from("Put the computer to sleep"),
                command: Box::from("systemctl suspend"),
                icon: PathBuf::from("system-suspend").into_boxed_path(),
            },
            Self::Custom(c) => c,
        }
    }
}
