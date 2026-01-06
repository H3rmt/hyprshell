#![allow(clippy::print_stderr, clippy::print_stdout)]

use crate::util;
use anyhow::Context;
use core_lib::default;
use core_lib::ini::IniFile;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

pub fn check_class(class: Option<String>) -> anyhow::Result<()> {
    util::init_gtk();
    util::check_themes();
    util::reload_desktop_data().context("Failed to reload desktop data")?;
    util::reload_icons(false);
    windows_lib::reload_class_to_icon_map().context("Failed to reload class to icon map")?;
    debug!("prepared desktop files and icon map");

    if let Some(class) = class {
        println!("searching for {class}");
        check_icon(&class);
    } else {
        println!("no class provided, iterating over all clients");
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
    let desktop_files = core_lib::util::collect_desktop_files();
    for file in desktop_files {
        let Ok(content) = fs::read_to_string(file.path()) else {
            eprintln!("Failed to read desktop file: {}", file.path().display());
            continue;
        };
        let ini = IniFile::from_str(&content);
        println!(
            "{}: {} [Type={}] [Terminal={}] [NoDisplay={}]",
            file.path().display(),
            ini.get_section("Desktop Entry")
                .and_then(|s| s.get_first("Name"))
                .unwrap_or_default(),
            ini.get_section("Desktop Entry")
                .and_then(|s| s.get_first("Type"))
                .unwrap_or_default(),
            ini.get_section("Desktop Entry")
                .and_then(|s| s.get_first("Terminal"))
                .unwrap_or_default(),
            ini.get_section("Desktop Entry")
                .and_then(|s| s.get_first("NoDisplay"))
                .unwrap_or_default(),
        );
    }
}

pub fn search(text: &str, all: bool, config_file: &Path, data_dir: &Path) {
    let (plugins, max_items) = config_lib::load_and_migrate_config(config_file, true)
        .ok()
        .and_then(|c| c.windows)
        .and_then(|w| w.overview)
        .map_or_else(
            || {
                warn!("Failed to get plugins from config, falling back to default");
                (config_lib::Launcher::default().plugins, 5)
            },
            |o| (o.launcher.plugins, o.launcher.max_items),
        );
    launcher_lib::debug::get_matches(&plugins, text, all, max_items, data_dir);
}

pub fn info(
    data_dir: &Path,
    cache_dir: &Path,
    css_file: &Path,
    config_file: &Path,
    system_data_dir: &Path,
) {
    println!("config version: {}", config_lib::CURRENT_CONFIG_VERSION);

    println!("css_file: {}", css_file.display());
    println!("config_file: {}", config_file.display());
    println!("data_dir: {}", data_dir.display());
    println!("cache_dir: {}", cache_dir.display());
    println!("system_data_dir: {}", system_data_dir.display());

    let dirs = [
        ("data_dir", data_dir),
        ("cache_dir", cache_dir),
        ("system_data_dir", system_data_dir),
    ];

    for (name, path) in dirs {
        if path.exists() && path.is_dir() {
            let (folders, files) = count_dir(path);
            println!("{name}: {folders} folders, {files} files (recursive)");
        } else {
            println!(
                "{name}: not a directory or does not exist: {}",
                path.display()
            );
        }
    }
}

fn count_dir(path: &Path) -> (usize, usize) {
    let mut folders = 0usize;
    let mut files = 0usize;
    let mut stack = vec![path.to_path_buf()];
    while let Some(p) = stack.pop() {
        match fs::read_dir(&p) {
            Ok(rd) => {
                for entry in rd.flatten() {
                    let path = entry.path();
                    match entry.file_type() {
                        Ok(ft) if ft.is_dir() => {
                            folders += 1;
                            stack.push(path);
                        }
                        Ok(ft) if ft.is_file() => files += 1,
                        _ => {}
                    }
                }
            }
            Err(_) => {}
        }
    }
    (folders, files)
}
