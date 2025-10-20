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
            g_windows.scale.set_value(windows.scale);
            g_windows
                .items_per_row
                .set_value(f64::from(windows.items_per_row));
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

pub fn update_overview(
    g_overview: &GTKOverview,
    overview: Option<&Overview>,
    view_stack: &ViewStack,
) {
    match overview {
        Some(overview) => {
            g_overview.row.set_enable_expansion(true);
            g_overview.row.set_expanded(true);
            g_overview.key.set_text(&overview.key);
            g_overview.modifier.set_selected(match overview.modifier {
                config_lib::Modifier::Alt => 0,
                config_lib::Modifier::Ctrl => 1,
                config_lib::Modifier::Super => 2,
            });
            update_windows_filter(&g_overview.filter, &overview.filter_by);
            g_overview.hide_filtered.set_active(overview.hide_filtered);

            update_launcher(&g_overview.launcher, Some(&overview.launcher), view_stack);
        }
        None => {
            g_overview.row.set_enable_expansion(false);
            update_launcher(&g_overview.launcher, None, view_stack);
        }
    }
}

pub fn update_windows_filter(g_filter: &GTKWindowsFilter, filter: &Vec<FilterBy>) {
    g_filter
        .same_class
        .set_active(filter.contains(&FilterBy::SameClass));
    g_filter
        .workspace
        .set_active(filter.contains(&FilterBy::CurrentWorkspace));
    g_filter
        .monitor
        .set_active(filter.contains(&FilterBy::CurrentMonitor));
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
            g_swich.modifier.set_selected(match switch.modifier {
                config_lib::Modifier::Alt => 0,
                config_lib::Modifier::Ctrl => 1,
                config_lib::Modifier::Super => 2,
            });
            update_windows_filter(&g_swich.filter, &switch.filter_by);
            g_swich
                .switch_workspaces
                .set_active(switch.switch_workspaces);
        }
        None => {
            g_swich.row.set_enable_expansion(false);
        }
    }
}

pub fn update_launcher(
    gtk_config: &GTKLauncher,
    config: Option<&Launcher>,
    view_stack: &ViewStack,
) {
    match config {
        Some(launcher) => {
            trace!("Adding launcher view");
            if view_stack.child_by_name("launcher").is_none() {
                view_stack.add_titled_with_icon(
                    &gtk_config.view,
                    Some("launcher"),
                    "Launcher",
                    "configure",
                );
            }
            gtk_config.row.set_enable_expansion(true);
            gtk_config.row.set_expanded(true);
            gtk_config
                .modifier
                .set_selected(match launcher.launch_modifier {
                    config_lib::Modifier::Alt => 0,
                    config_lib::Modifier::Ctrl => 1,
                    config_lib::Modifier::Super => 2,
                });
            gtk_config.width.set_value(launcher.width as f64);
            gtk_config.max_items.set_value(launcher.max_items as f64);
            gtk_config
                .show_when_empty
                .set_active(launcher.show_when_empty);
            match &launcher.default_terminal {
                Some(terminal) => {
                    gtk_config.dont_use_default_terminal.set_active(true);
                    gtk_config.terminal.set_text(&terminal);
                    gtk_config.terminal.set_sensitive(true);
                }
                None => {
                    gtk_config.dont_use_default_terminal.set_active(false);
                    gtk_config.terminal.set_text("");
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

pub fn update_plugins(gtk_config: &GTKPlugins, config: Option<&Plugins>) {
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
                    gtk_config
                        .applications
                        .cache_weeks
                        .set_value(applications.run_cache_weeks as f64);
                    gtk_config
                        .applications
                        .submenu
                        .set_active(applications.show_actions_submenu);
                    gtk_config
                        .applications
                        .show_exec
                        .set_active(applications.show_execs);
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
