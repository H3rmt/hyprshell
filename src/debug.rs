use crate::util;
use anyhow::Context;
use config_lib::ApplicationsPluginConfig;
use core_lib::default;
use std::path::Path;
use tracing::debug;

pub fn check_class(class: Option<String>) -> anyhow::Result<()> {
    util::init_gtk();
    util::check_themes();
    util::reload_desktop_data().context("Failed to reload desktop data")?;
    util::reload_icons(false);
    windows_lib::reload_class_to_icon_map().context("Failed to reload class to icon map")?;
    debug!("prepared desktop files and icon map");

    if let Some(class) = class {
        debug!("searching for {class}");
        check_icon(&class);
    } else {
        debug!("no class provided, iterating over all clients");
        for client in exec_lib::get_clients() {
            let class = client.class;
            debug!("checking {class}");
            check_icon(&class);
        }
    }
    Ok(())
}

fn check_icon(class: &str) {
    let in_theme = default::theme_has_icon_name(class);
    println!(
        "Icon ({class}) {} in theme (first choice)",
        if in_theme { "is" } else { "is not" }
    );
    let icon = windows_lib::get_icon_name_by_name_from_desktop_files(class);
    println!(
        "Icon ({class}) {} in desktop files (second choice) {}",
        if icon.is_some() { "is" } else { "is not" },
        if let Some(icon) = icon {
            format!("{:?} [icon: {}]", icon.2, icon.0.display())
        } else {
            String::new()
        }
    );
}

pub fn list_icons() -> anyhow::Result<()> {
    util::init_gtk();
    util::check_themes();
    util::reload_icons(false);
    let icons = default::get_all_icons().context("Failed to get icons")?;
    for icon in icons.iter() {
        println!("{icon}");
    }
    drop(icons);
    Ok(())
}

pub fn list_desktop_files() {
    let desktop_files = core_lib::collect_desktop_files();
    for file in desktop_files {
        println!("{}", file.path().display());
    }
}

pub fn search(text: &str, all: bool, config_path: &Path, data_dir: &Path) {
    let (plugins, max_items) = config_lib::load_and_migrate_config(config_path, true)
        .ok()
        .and_then(|c| c.windows)
        .and_then(|w| w.overview)
        .map_or_else(
            || {
                (
                    config_lib::Plugins {
                        applications: Some(ApplicationsPluginConfig::default()),
                        shell: None,
                        terminal: None,
                        websearch: None,
                        calc: None,
                        path: None,
                    },
                    5,
                )
            },
            |o| (o.launcher.plugins, o.launcher.max_items),
        );
    launcher_lib::debug::get_matches(&plugins, text, all, max_items, data_dir);
}
