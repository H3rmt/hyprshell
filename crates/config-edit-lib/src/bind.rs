use crate::structs::GTKConfig;
use config_lib::{Config, Windows};

use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{info, trace};

pub fn update_config(gtk_config: &GTKConfig, config: &Config) {
    match &config.windows {
        Some(windows) => {
            gtk_config.windows.enabled.set_active(true);
            gtk_config.windows.scale.set_value(windows.scale);
            gtk_config
                .windows
                .items_per_row
                .set_value(f64::from(windows.items_per_row));
        }
        None => {
            gtk_config.windows.enabled.set_active(false);
        }
    }
}

pub fn bind(gtk_config: GTKConfig, config: Config) {
    let config = Rc::new(RefCell::new(config));
    let gtk_config = Rc::new(RefCell::new(gtk_config));
    let gtk_conf = gtk_config.clone();
    let gtk_conf = gtk_conf.borrow();

    let config_clone = config.clone();
    let gtk_config_clone = gtk_config;
    gtk_conf.windows.enabled.connect_toggled(move |button| {
        if button.is_active() {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                c.windows = Some(Windows::default());
            }
            // ensure that all inputs show the data from default
            let cfg = gtk_config_clone.borrow();
            update_config(&cfg, &config_clone.borrow());
            cfg.windows.view.set_visible(true);
        } else {
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                c.windows = None;
            }
            let cfg = gtk_config_clone.borrow();
            cfg.windows.view.set_visible(false);
        }
    });

    // Scale
    let config_clone = config.clone();
    gtk_conf.windows.scale.connect_value_changed(move |button| {
        trace!("Scale changed to {}", button.value());
        if let Ok(mut c) = config_clone.try_borrow_mut() {
            if let Some(windows) = c.windows.as_mut() {
                windows.scale = (button.value() * 100.0).round() / 100.0;
            }
        }
    });

    // Items per row
    let config_clone = config.clone();
    gtk_conf
        .windows
        .items_per_row
        .connect_value_changed(move |button| {
            trace!("Items per row changed to {}", button.value());
            if let Ok(mut c) = config_clone.try_borrow_mut() {
                if let Some(windows) = c.windows.as_mut() {
                    windows.items_per_row = button.value() as u8;
                }
            }
        });

    let config_clone = config;
    gtk_conf.save.connect_clicked(move |_button| {
        let c = config_clone.borrow();
        info!("{c:#?}");
    });
}
