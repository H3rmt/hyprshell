use crate::structs::{
    GTKConfig, GTKLauncher, GTKOverview, GTKPlugins, GTKSwitch, GTKWindowsFilter,
};
use adw::ViewStack;
use adw::prelude::{EditableExt, ExpanderRowExt, PreferencesRowExt, WidgetExt};
use config_lib::{Config, FilterBy, Launcher, Overview, Plugins, Switch};
use tracing::trace;

pub fn update_config(gtk_config: &GTKConfig, config: &Config) {
    let view_stack = &gtk_config.view_stack;
    let g_windows = &gtk_config.windows;
    match &config.windows {
        Some(windows) => {
            g_windows.row.set_enable_expansion(true);
            g_windows.row.set_expanded(true);
            if g_windows.scale.value() != windows.scale {
                g_windows.scale.set_value(windows.scale);
            }
            if g_windows.items_per_row.value() as u8 != windows.items_per_row {
                g_windows
                    .items_per_row
                    .set_value(f64::from(windows.items_per_row));
            }
            update_overview(&g_windows.overview, windows.overview.as_ref(), view_stack);
            update_switch(&g_windows.switch, windows.switch.as_ref());
        }
        None => {
            gtk_config.windows.row.set_enable_expansion(false);
            update_overview(&g_windows.overview, None, view_stack);
            update_switch(&g_windows.switch, None);
        }
    }
}

fn update_overview(g_overview: &GTKOverview, overview: Option<&Overview>, view_stack: &ViewStack) {
    match overview {
        Some(overview) => {
            g_overview.row.set_enable_expansion(true);
            g_overview.row.set_expanded(true);
            if &g_overview.key.text() != &*overview.key {
                g_overview.key.set_text(&overview.key);
            }
            let desired_modifier = match overview.modifier {
                config_lib::Modifier::Alt => 0,
                config_lib::Modifier::Ctrl => 1,
                config_lib::Modifier::Super => 2,
            };
            if g_overview.modifier.selected() != desired_modifier {
                g_overview.modifier.set_selected(desired_modifier);
            }
            update_windows_filter(&g_overview.filter, &overview.filter_by);
            if g_overview.hide_filtered.is_active() != overview.hide_filtered {
                g_overview.hide_filtered.set_active(overview.hide_filtered);
            }

            update_launcher(&g_overview.launcher, Some(&overview.launcher), view_stack);
        }
        None => {
            g_overview.row.set_enable_expansion(false);
            update_launcher(&g_overview.launcher, None, view_stack);
        }
    }
}

fn update_windows_filter(g_filter: &GTKWindowsFilter, filter: &Vec<FilterBy>) {
    if g_filter.same_class.is_active() != filter.contains(&FilterBy::SameClass) {
        g_filter
            .same_class
            .set_active(filter.contains(&FilterBy::SameClass));
    }
    if g_filter.workspace.is_active() != filter.contains(&FilterBy::CurrentWorkspace) {
        g_filter
            .workspace
            .set_active(filter.contains(&FilterBy::CurrentWorkspace));
    }
    if g_filter.monitor.is_active() != filter.contains(&FilterBy::CurrentMonitor) {
        g_filter
            .monitor
            .set_active(filter.contains(&FilterBy::CurrentMonitor));
    }
    g_filter.row.set_title(&if filter.len() == 0 {
        String::from("Filter")
    } else if filter.len() == 1 {
        format!("Filter: {:?}", filter[0])
    } else if filter.len() == 2 {
        format!("Filter: {:?} + {:?}", filter[0], filter[1])
    } else {
        // should not be possible, maybe if loaded from config
        format!(
            "Filter: {:?} + {:?} + {:?}",
            filter[0], filter[1], filter[2]
        )
    })
}

fn update_switch(g_swich: &GTKSwitch, switch: Option<&Switch>) {
    match switch {
        Some(switch) => {
            g_swich.row.set_enable_expansion(true);
            g_swich.row.set_expanded(true);
            let desired_modifier = match switch.modifier {
                config_lib::Modifier::Alt => 0,
                config_lib::Modifier::Ctrl => 1,
                config_lib::Modifier::Super => 2,
            };
            if g_swich.modifier.selected() != desired_modifier {
                g_swich.modifier.set_selected(desired_modifier);
            }
            update_windows_filter(&g_swich.filter, &switch.filter_by);
            if g_swich.switch_workspaces.is_active() != switch.switch_workspaces {
                g_swich
                    .switch_workspaces
                    .set_active(switch.switch_workspaces);
            }
        }
        None => {
            g_swich.row.set_enable_expansion(false);
        }
    }
}

fn update_launcher(gtk_config: &GTKLauncher, config: Option<&Launcher>, view_stack: &ViewStack) {
    match config {
        Some(launcher) => {
            if view_stack.child_by_name("launcher").is_none() {
                trace!("Adding launcher view");
                view_stack.add_titled_with_icon(
                    &gtk_config.view,
                    Some("launcher"),
                    "Launcher",
                    "configure",
                );
            }
            gtk_config.row.set_enable_expansion(true);
            gtk_config.row.set_expanded(true);
            let desired_modifier = match launcher.launch_modifier {
                config_lib::Modifier::Alt => 0,
                config_lib::Modifier::Ctrl => 1,
                config_lib::Modifier::Super => 2,
            };
            if gtk_config.modifier.selected() != desired_modifier {
                gtk_config.modifier.set_selected(desired_modifier);
            }
            if gtk_config.width.value() as u32 != launcher.width {
                gtk_config.width.set_value(launcher.width as f64);
            }
            if gtk_config.max_items.value() as u8 != launcher.max_items {
                gtk_config.max_items.set_value(launcher.max_items as f64);
            }
            if gtk_config.show_when_empty.is_active() != launcher.show_when_empty {
                gtk_config
                    .show_when_empty
                    .set_active(launcher.show_when_empty);
            }
            match &launcher.default_terminal {
                Some(terminal) => {
                    if !gtk_config.dont_use_default_terminal.is_active() {
                        gtk_config.dont_use_default_terminal.set_active(true);
                    }
                    if &*gtk_config.terminal.text() != &**terminal {
                        gtk_config.terminal.set_text(&terminal);
                    }
                    gtk_config.terminal.set_sensitive(true);
                }
                None => {
                    if gtk_config.dont_use_default_terminal.is_active() {
                        gtk_config.dont_use_default_terminal.set_active(false);
                    }
                    if gtk_config.terminal.text() != "" {
                        gtk_config.terminal.set_text("");
                    }
                    gtk_config.terminal.set_sensitive(false);
                }
            }
            update_plugins(&gtk_config.plugins, Some(&launcher.plugins));
        }
        None => {
            trace!("Removing launcher view");
            if view_stack.child_by_name("launcher").is_some() {
                view_stack.remove(&gtk_config.view);
            }
        }
    }
}

fn update_plugins(gtk_config: &GTKPlugins, config: Option<&Plugins>) {
    match config {
        Some(plugins) => {
            gtk_config.row.set_enable_expansion(true);
            gtk_config.row.set_expanded(true);

            gtk_config.terminal.set_active(plugins.terminal.is_some());
            gtk_config.shell.set_active(plugins.shell.is_some());
            gtk_config.calc.set_active(plugins.calc.is_some());
            gtk_config.path.set_active(plugins.path.is_some());

            match &plugins.applications {
                Some(applications) => {
                    gtk_config.applications.row.set_enable_expansion(true);
                    gtk_config.applications.row.set_expanded(true);
                    if gtk_config.applications.cache_weeks.value() as u8
                        != applications.run_cache_weeks
                    {
                        gtk_config
                            .applications
                            .cache_weeks
                            .set_value(applications.run_cache_weeks as f64);
                    }
                    if gtk_config.applications.submenu.is_active()
                        != applications.show_actions_submenu
                    {
                        gtk_config
                            .applications
                            .submenu
                            .set_active(applications.show_actions_submenu);
                    }
                    if gtk_config.applications.show_exec.is_active() != applications.show_execs {
                        gtk_config
                            .applications
                            .show_exec
                            .set_active(applications.show_execs);
                    }
                }
                None => {
                    gtk_config.applications.row.set_enable_expansion(false);
                }
            }
        }
        None => {
            gtk_config.row.set_enable_expansion(false);
            gtk_config.row.set_expanded(false);
        }
    }
}
