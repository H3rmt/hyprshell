use crate::structs::{GTKConfig, GTKOverviewFilter};
use config_lib::{Config, FilterBy, Overview, Windows};

use adw::prelude::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use tracing::{info, trace};

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
            match &windows.overview {
                Some(overview) => {
                    let g_overview = &gtk_config.windows.overview;
                    g_overview.row.set_enable_expansion(true);
                    g_overview.row.set_expanded(true);
                    g_overview.key.set_text(&overview.key);
                    g_overview.modifier.set_selected(match overview.modifier {
                        config_lib::Modifier::Alt => 0,
                        config_lib::Modifier::Ctrl => 1,
                        config_lib::Modifier::Super => 2,
                    });

                    update_overview_filter(&g_overview.filter, &overview.filter_by);

                    gtk_config
                        .windows
                        .overview
                        .hide_filtered
                        .set_active(overview.hide_filtered);
                }
                None => {
                    gtk_config.windows.overview.row.set_enable_expansion(false);
                }
            }
        }
        None => {
            gtk_config.windows.row.set_enable_expansion(false);
        }
    }
}

fn update_overview_filter(g_filter: &GTKOverviewFilter, filter: &Vec<FilterBy>) {
    g_filter
        .same_class
        .set_active(filter.contains(&FilterBy::SameClass));
    g_filter
        .workspace
        .set_active(filter.contains(&FilterBy::CurrentWorkspace));
    g_filter
        .monitor
        .set_active(filter.contains(&FilterBy::CurrentMonitor));
    g_filter.row.set_expanded(filter.len() > 0);
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

pub fn bind(gtk_config: GTKConfig, config: Config) {
    let config = Rc::new(RefCell::new(config));
    let gtk_config = Rc::new(RefCell::new(gtk_config));
    let gtk_conf = gtk_config.clone();
    let gtk_conf = gtk_conf.borrow();

    let config_clone = config.clone();
    gtk_conf.save.connect_clicked(move |_button| {
        let c = config_clone.borrow();
        info!("{c:#?}");
    });

    bind_windows(gtk_conf, gtk_config, config.clone());
}

fn bind_windows(
    gtk_conf: Ref<GTKConfig>,
    gtk_config: Rc<RefCell<GTKConfig>>,
    config: Rc<RefCell<Config>>,
) {
    let windows = &gtk_conf.windows;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    windows.row.connect_enable_expansion_notify(move |button| {
        trace!("windows.row changed to {}", button.enables_expansion());
        if button.enables_expansion() {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                c.windows = Some(Windows::default());
            }
            // ensure that all inputs show the data from default
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        } else {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                c.windows = None;
            }
        }
    });

    // Scale
    let config_clone = config.clone();
    windows.scale.connect_value_changed(move |button| {
        trace!("windows.scale changed to {}", button.value());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.scale = (button.value() * 100.0).round() / 100.0;
            }
        }
    });

    // Items per row
    let config_clone = config.clone();
    windows.items_per_row.connect_value_changed(move |button| {
        trace!("windows.items_per_row changed to {}", button.value());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.items_per_row = button.value() as u8;
            }
        }
    });

    bind_overview(gtk_conf, gtk_config.clone(), config.clone());
}

fn bind_overview(
    gtk_conf: Ref<GTKConfig>,
    gtk_config: Rc<RefCell<GTKConfig>>,
    config: Rc<RefCell<Config>>,
) {
    let overview = &gtk_conf.windows.overview;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    overview.row.connect_enable_expansion_notify(move |button| {
        trace!(
            "windows.overview.row changed to {}",
            button.enables_expansion()
        );
        if button.enables_expansion() {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    windows.overview = Some(Overview::default());
                }
            }
            // ensure that all inputs show the data from default
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        } else {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    windows.overview = None
                }
            }
        }
    });

    let config_clone = config.clone();
    overview.key.connect_text_notify(move |entry| {
        trace!("windows.overview.key changed to {}", entry.text());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.key = entry.text().to_string().into_boxed_str();
                }
            }
        }
    });

    let config_clone = config.clone();
    overview.modifier.connect_selected_notify(move |dropdown| {
        trace!(
            "windows.overview.modifier changed to {}",
            dropdown.selected(),
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.modifier = match dropdown.selected() {
                        0 => config_lib::Modifier::Alt,
                        1 => config_lib::Modifier::Ctrl,
                        2 => config_lib::Modifier::Super,
                        _ => panic!("Invalid modifier selected"),
                    }
                }
            }
        }
    });

    bind_overview_filter(&overview.filter, gtk_config.clone(), config.clone());

    let config_clone = config.clone();
    overview.hide_filtered.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.hide_filtered changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.hide_filtered = entry.is_active();
                }
            }
        }
    });
}

fn bind_overview_filter(
    filter: &GTKOverviewFilter,
    gtk_config: Rc<RefCell<GTKConfig>>,
    config: Rc<RefCell<Config>>,
) {
    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.same_class.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.filter.same_class changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    if entry.is_active() {
                        overview.filter_by.push(FilterBy::SameClass);
                    } else {
                        overview.filter_by.retain(|f| *f != FilterBy::SameClass);
                    }
                    // use update function to update other parts of ui
                    update_overview_filter(
                        &gtk_config_clone.borrow().windows.overview.filter,
                        &overview.filter_by,
                    )
                }
            }
        }
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.workspace.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.filter.workspace changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    if entry.is_active() {
                        overview.filter_by.push(FilterBy::CurrentWorkspace);
                        // current monitor and current workspace are mutually exclusive (not really, but it doesn't make sense)
                        if overview.filter_by.contains(&FilterBy::CurrentMonitor) {
                            overview
                                .filter_by
                                .retain(|f| *f != FilterBy::CurrentMonitor);
                        }
                    } else {
                        overview
                            .filter_by
                            .retain(|f| *f != FilterBy::CurrentWorkspace);
                    }
                    // use update function to update other parts of ui
                    update_overview_filter(
                        &gtk_config_clone.borrow().windows.overview.filter,
                        &overview.filter_by,
                    )
                }
            }
        }
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.monitor.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.filter.monitor changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    if entry.is_active() {
                        overview.filter_by.push(FilterBy::CurrentMonitor);
                        // current monitor and current workspace are mutually exclusive (not really, but it doesn't make sense)
                        if overview.filter_by.contains(&FilterBy::CurrentWorkspace) {
                            overview
                                .filter_by
                                .retain(|f| *f != FilterBy::CurrentWorkspace);
                        }
                    } else {
                        overview
                            .filter_by
                            .retain(|f| *f != FilterBy::CurrentMonitor);
                    }
                    // use update function to update other parts of ui
                    update_overview_filter(
                        &gtk_config_clone.borrow().windows.overview.filter,
                        &overview.filter_by,
                    )
                }
            }
        }
    });
}
