#![allow(clippy::too_many_lines)]

use crate::structs::{GTKConfig, GTKWindowsFilter};
use config_lib::{
    ApplicationsPluginConfig, Config, EmptyConfig, FilterBy, Overview, Switch, Windows,
};

use crate::update::update_config;
use crate::update_changes_view::set_previous_config;
use adw::AlertDialog;
use adw::prelude::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use tracing::{debug, trace, warn};

pub fn bind(gtk_config: GTKConfig, config: Config) {
    let config = Rc::new(RefCell::new(config));
    let gtk_config = Rc::new(RefCell::new(gtk_config));
    let gtk_conf = gtk_config.clone();
    let gtk_conf = gtk_conf.borrow();

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    gtk_conf.save.connect_clicked(move |_button| {
        let c = config_clone.borrow();
        if let Err(err) = config_lib::write_config(&gtk_config_clone.borrow().path, &c, true) {
            warn!("Failed to save config: {err:?}");
            let dialog = AlertDialog::builder()
                .heading("Failed to save config")
                .body(format!("{err:#}"))
                .close_response("close")
                .build();
            dialog.add_responses(&[("close", "Close")]);
            return;
        }
        debug!("{c:#?}");
        set_previous_config(c.clone());
        update_config(&gtk_config_clone.borrow(), &c);
    });

    bind_windows(&gtk_conf, &gtk_config, &config);
}

fn bind_windows(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
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
        } else if let Ok(mut c) = config_clone.try_borrow_mut() {
            c.windows = None;
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    windows.scale.connect_value_changed(move |button| {
        trace!("windows.scale changed to {}", button.value());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.scale = (button.value() * 100.0).round() / 100.0;
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    windows.items_per_row.connect_value_changed(move |button| {
        trace!("windows.items_per_row changed to {}", button.value());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                #[allow(clippy::cast_sign_loss)]
                {
                    windows.items_per_row = button.value() as u8;
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_overview(gtk_conf, gtk_config, config);
    bind_switch(gtk_conf, gtk_config, config);
}

fn bind_overview(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
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
        } else if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.overview = None;
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    overview.key.connect_text_notify(move |entry| {
        trace!("windows.overview.key changed to {}", entry.text());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.key = entry.text().to_string().into_boxed_str();
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
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
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_overview_filter(&overview.filter, gtk_config, config);

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
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
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_launcher(gtk_conf, gtk_config, config);
}

fn bind_overview_filter(
    filter: &GTKWindowsFilter,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
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
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
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
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
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
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });
}

fn bind_switch(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
) {
    let switch = &gtk_conf.windows.switch;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    switch.row.connect_enable_expansion_notify(move |button| {
        trace!(
            "windows.switch.row changed to {}",
            button.enables_expansion()
        );
        if button.enables_expansion() {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    windows.switch = Some(Switch::default());
                }
            }
        } else if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.switch = None;
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    switch.modifier.connect_selected_notify(move |dropdown| {
        trace!("windows.switch.modifier changed to {}", dropdown.selected(),);
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(switch) = windows.switch.as_mut() {
                    switch.modifier = match dropdown.selected() {
                        0 => config_lib::Modifier::Alt,
                        1 => config_lib::Modifier::Ctrl,
                        2 => config_lib::Modifier::Super,
                        _ => panic!("Invalid modifier selected"),
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_switch_filter(&switch.filter, gtk_config, config);

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    switch
        .switch_workspaces
        .connect_active_notify(move |entry| {
            trace!(
                "windows.switch.switch_workspaces changed to {}",
                entry.is_active()
            );
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    if let Some(switch) = windows.switch.as_mut() {
                        switch.switch_workspaces = entry.is_active();
                    }
                }
            }
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        });
}

fn bind_switch_filter(
    filter: &GTKWindowsFilter,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
) {
    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.same_class.connect_active_notify(move |entry| {
        trace!(
            "windows.switch.filter.same_class changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(switch) = windows.switch.as_mut() {
                    if entry.is_active() {
                        switch.filter_by.push(FilterBy::SameClass);
                    } else {
                        switch.filter_by.retain(|f| *f != FilterBy::SameClass);
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.workspace.connect_active_notify(move |entry| {
        trace!(
            "windows.switch.filter.workspace changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(switch) = windows.switch.as_mut() {
                    if entry.is_active() {
                        switch.filter_by.push(FilterBy::CurrentWorkspace);
                        // current monitor and current workspace are mutually exclusive (not really, but it doesn't make sense)
                        if switch.filter_by.contains(&FilterBy::CurrentMonitor) {
                            switch.filter_by.retain(|f| *f != FilterBy::CurrentMonitor);
                        }
                    } else {
                        switch
                            .filter_by
                            .retain(|f| *f != FilterBy::CurrentWorkspace);
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    filter.monitor.connect_active_notify(move |entry| {
        trace!(
            "windows.switch.filter.monitor changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(switch) = windows.switch.as_mut() {
                    if entry.is_active() {
                        switch.filter_by.push(FilterBy::CurrentMonitor);
                        // current monitor and current workspace are mutually exclusive (not really, but it doesn't make sense)
                        if switch.filter_by.contains(&FilterBy::CurrentWorkspace) {
                            switch
                                .filter_by
                                .retain(|f| *f != FilterBy::CurrentWorkspace);
                        }
                    } else {
                        switch.filter_by.retain(|f| *f != FilterBy::CurrentMonitor);
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });
}

fn bind_launcher(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
) {
    let launcher = &gtk_conf.windows.overview.launcher;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher.modifier.connect_selected_notify(move |dropdown| {
        trace!(
            "windows.overview.launcher.modifier changed to {}",
            dropdown.selected(),
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.launch_modifier = match dropdown.selected() {
                        0 => config_lib::Modifier::Alt,
                        1 => config_lib::Modifier::Ctrl,
                        2 => config_lib::Modifier::Super,
                        _ => panic!("Invalid modifier selected"),
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher.max_items.connect_value_changed(move |button| {
        trace!(
            "windows.overview.launcher.max_items changed to {}",
            button.value()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        overview.launcher.max_items =
                            ((button.value() * 100.0).round() / 100.0) as u8;
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher.width.connect_value_changed(move |button| {
        trace!(
            "windows.overview.launcher.width changed to {}",
            button.value()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        overview.launcher.width = ((button.value() * 100.0).round() / 100.0) as u32;
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher
        .show_when_empty
        .connect_active_notify(move |entry| {
            trace!(
                "windows.overview.launcher.show_when_empty changed to {}",
                entry.is_active()
            );
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    if let Some(overview) = windows.overview.as_mut() {
                        overview.launcher.show_when_empty = entry.is_active();
                    }
                }
            }
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher
        .dont_use_default_terminal
        .connect_active_notify(move |entry| {
            trace!(
                "windows.overview.launcher.dont_use_default_terminal changed to {}",
                entry.is_active()
            );
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    if let Some(overview) = windows.overview.as_mut() {
                        overview.launcher.default_terminal = if entry.is_active() {
                            Some(Box::from(""))
                        } else {
                            None
                        };
                    }
                }
            }
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    launcher.terminal.connect_text_notify(move |button| {
        trace!(
            "windows.overview.launcher.terminal changed to {}",
            button.text()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.default_terminal = Some(Box::from(button.text()));
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_plugins(gtk_conf, gtk_config, config);
}

fn bind_plugins(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
) {
    let plugins = &gtk_conf.windows.overview.launcher.plugins;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    plugins.terminal.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.terminal changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.plugins.terminal = if entry.is_active() {
                        Some(EmptyConfig::default())
                    } else {
                        None
                    };
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    plugins.shell.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.shell changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.plugins.shell = if entry.is_active() {
                        Some(EmptyConfig::default())
                    } else {
                        None
                    };
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    plugins.calc.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.calc changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.plugins.calc = if entry.is_active() {
                        Some(EmptyConfig::default())
                    } else {
                        None
                    };
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    plugins.path.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.path changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    overview.launcher.plugins.path = if entry.is_active() {
                        Some(EmptyConfig::default())
                    } else {
                        None
                    };
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    bind_application_plugin(gtk_conf, gtk_config, config);
}

fn bind_application_plugin(
    gtk_conf: &Ref<GTKConfig>,
    gtk_config: &Rc<RefCell<GTKConfig>>,
    config: &Rc<RefCell<Config>>,
) {
    let applications = &gtk_conf.windows.overview.launcher.plugins.applications;

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    applications
        .row
        .connect_enable_expansion_notify(move |button| {
            trace!(
                "windows.overview.launcher.plugins.applications.row changed to {}",
                button.enables_expansion()
            );
            if button.enables_expansion() {
                if let Ok(mut c) = config_clone.try_borrow_mut() {
                    if let Some(windows) = c.windows.as_mut() {
                        if let Some(overview) = windows.overview.as_mut() {
                            overview.launcher.plugins.applications =
                                Some(ApplicationsPluginConfig::default());
                        }
                    }
                }
            } else if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    if let Some(overview) = windows.overview.as_mut() {
                        overview.launcher.plugins.applications = None;
                    }
                }
            }
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    applications
        .cache_weeks
        .connect_value_changed(move |entry| {
            trace!(
                "windows.overview.launcher.plugins.applications.cache_weeks changed to {}",
                entry.value()
            );
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    if let Some(overview) = windows.overview.as_mut() {
                        if let Some(applications) = overview.launcher.plugins.applications.as_mut()
                        {
                            #[allow(clippy::cast_sign_loss)]
                            {
                                applications.run_cache_weeks =
                                    ((entry.value() * 100.0).round() / 100.0) as u8;
                            }
                        }
                    }
                }
            }
            update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
        });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    applications.show_exec.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.applications.show_exec changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    if let Some(applications) = overview.launcher.plugins.applications.as_mut() {
                        applications.show_execs = entry.is_active();
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config.clone();
    applications.submenu.connect_active_notify(move |entry| {
        trace!(
            "windows.overview.launcher.plugins.applications.submenu changed to {}",
            entry.is_active()
        );
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                if let Some(overview) = windows.overview.as_mut() {
                    if let Some(applications) = overview.launcher.plugins.applications.as_mut() {
                        applications.show_actions_submenu = entry.is_active();
                    }
                }
            }
        }
        update_config(&gtk_config_clone.borrow(), &config_clone.borrow());
    });
}
