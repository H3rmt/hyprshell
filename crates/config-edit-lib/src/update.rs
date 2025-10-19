use crate::structs::{GTKConfig, GTKOverview, GTKSwitch, GTKWindowsFilter};
use adw::prelude::{EditableExt, ExpanderRowExt, PreferencesRowExt};
use config_lib::{Config, FilterBy, Overview, Switch};

pub fn update_config(gtk_config: &GTKConfig, config: &Config) {
    match &config.windows {
        Some(windows) => {
            let g_windows = &gtk_config.windows;
            g_windows.row.set_enable_expansion(true);
            g_windows.row.set_expanded(true);
            g_windows.scale.set_value(windows.scale);
            g_windows
                .items_per_row
                .set_value(f64::from(windows.items_per_row));
            update_overview(&g_windows.overview, windows.overview.as_ref());
            update_switch(&g_windows.switch, windows.switch.as_ref());
        }
        None => {
            gtk_config.windows.row.set_enable_expansion(false);
        }
    }
}

fn update_overview(g_overview: &GTKOverview, overview: Option<&Overview>) {
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
        }
        None => {
            g_overview.row.set_enable_expansion(false);
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
