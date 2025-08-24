use crate::util;
use anyhow::{Context, bail};
use core_lib::default::get_all_mime_files;
use core_lib::{IniFile, get_config_home};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use tracing::{debug, warn};

pub fn get(mime: &str) -> anyhow::Result<()> {
    util::reload_desktop_data().context("Failed to reload desktop data")?;

    let mut mimes = vec![];
    for (file, ini) in get_all_mime_files()
        .context("unable to get all mimefiles")?
        .iter()
    {
        let default = ini.get_section("Default Applications").and_then(|section| {
            section.get_first(mime).or_else(|| {
                ini.get_section("Added Associations")
                    .and_then(|section| section.get_first(mime))
            })
        });
        if let Some(default) = default {
            mimes.push((
                default.to_string(),
                file.path().to_string_lossy().to_string(),
            ));
        }
    }
    if mimes.is_empty() {
        bail!("No default application found for {mime}")
    }

    for (value, path) in mimes {
        println!("{mime}: {value} [{path}]");
    }
    Ok(())
}

pub fn set_default(mime: &str, value: &str) -> anyhow::Result<()> {
    let desktop_files = core_lib::collect_desktop_files();

    // check if valid desktop file
    let file = desktop_files.iter().find(|f| f.file_name() == value);
    match file {
        Some(file) => println!(
            "Desktop file {} at {}",
            file.file_name().display(),
            file.path().display()
        ),
        None => bail!("Invalid desktop file: {value}"),
    };

    let mut file = get_config_home();
    file.push("mimeapps.list");
    let str = if file.exists() {
        read_to_string(&file).with_context(|| format!("Failed to read file {}", file.display()))?
    } else {
        String::new()
    };
    let mut ini = IniFile::from_str(&str);
    let section = ini.section_entry("Default Applications").or_default();
    section.set_items(mime, vec![value]);

    let str = ini.format();
    write(&file, str).with_context(|| format!("Failed to write file {:?}", file.display()))?;

    println!("added {mime}: {value} to {:?}", file.display());
    Ok(())
}

pub fn add_association(mime: &str, value: &str) -> anyhow::Result<()> {
    let desktop_files = core_lib::collect_desktop_files();

    // check if valid desktop file
    let file = desktop_files.iter().find(|f| f.file_name() == value);
    match file {
        Some(file) => println!(
            "Desktop file {} at {}",
            file.file_name().display(),
            file.path().display()
        ),
        None => bail!("Invalid desktop file: {value}"),
    };

    let mut file = get_config_home();
    file.push("mimeapps.list");
    let str = if file.exists() {
        read_to_string(&file).with_context(|| format!("Failed to read file {}", file.display()))?
    } else {
        String::new()
    };
    let mut ini = IniFile::from_str(&str);
    let section = ini.section_entry("Added Associations").or_default();
    section.insert_item_at_front(mime, value);

    let str = ini.format();
    write(&file, str).with_context(|| format!("Failed to write file {:?}", file.display()))?;

    println!("added {mime}: {value} to {:?}", file.display());
    Ok(())
}

const USED_MIME_TYPES: &[&str] = &["x-scheme-handler/https", "inode/directory"];

pub fn list(all: bool) {
    let mime_files = core_lib::collect_mime_files();

    let mut mimes = HashMap::new();
    for file in mime_files {
        if let Ok(str) = read_to_string(file.path()) {
            let ini = IniFile::from_str(&str);
            debug!("mimeapps.list: {:?}", file.path());
            if let Some(section) = ini.get_section("Default Applications") {
                for (mime, values) in section {
                    let mut values = values.iter().map(ToString::to_string).collect::<Vec<_>>();
                    mimes
                        .entry(mime.to_string())
                        .or_insert((vec![], vec![]))
                        .0
                        .append(&mut values);
                }
            }
            if let Some(section) = ini.get_section("Added Associations") {
                for (mime, values) in section {
                    let mut values = values.iter().map(ToString::to_string).collect::<Vec<_>>();
                    mimes
                        .entry(mime.to_string())
                        .or_insert((vec![], vec![]))
                        .1
                        .append(&mut values);
                }
            }
        } else {
            warn!("Failed to read file: {:?}", file.path());
        }
    }

    if all {
        let mut mimes = mimes.into_iter().collect::<Vec<_>>();
        mimes.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (mime, (defaults, added)) in mimes {
            println!("{mime}: defaults: {defaults:?}, added: {added:?}");
        }
    } else {
        for mime in USED_MIME_TYPES {
            if let Some((defaults, added)) = mimes.get(*mime) {
                println!("{mime}: defaults: {defaults:?}, added: {added:?}");
            } else {
                println!("{mime}: <not set>");
            }
        }
    }
}

pub fn check() {
    let mime_files = core_lib::collect_mime_files();
    let desktop_files = core_lib::collect_desktop_files();

    let mut mime_files_map = HashMap::new();
    for file in mime_files {
        let mut default_mimes = HashMap::new();
        let mut added_mimes = HashMap::new();
        if let Ok(str) = read_to_string(file.path()) {
            let ini = IniFile::from_str(&str);
            if let Some(section) = ini.get_section("Default Applications") {
                debug!("mimeapps.list: {:?} Default Applications", file.path());
                for (mime, values) in section {
                    if default_mimes.contains_key(mime) {
                        warn!("{mime} already exists");
                    }
                    for value in values {
                        default_mimes.insert(mime.to_string(), (*value).to_string());
                    }
                }
            }
            if let Some(section) = ini.get_section("Added Associations") {
                debug!("mimeapps.list: {:?} Added Associations", file.path());
                for (mime, values) in section {
                    if added_mimes.contains_key(mime) {
                        warn!("{mime} already exists");
                    }
                    for value in values {
                        added_mimes.insert(mime.to_string(), (*value).to_string());
                    }
                }
            }
        } else {
            warn!("Failed to read file: {:?}", file.path());
        }
        mime_files_map.insert(file.path(), (default_mimes, added_mimes));
    }

    for (file, (default_mimes, added_mimes)) in mime_files_map {
        println!("{file:?}");
        let mut vec = default_mimes.iter().collect::<Vec<_>>();
        vec.sort_by_key(|&(mime, _)| mime.to_string());
        for (mime, value) in vec {
            if desktop_files.iter().any(|d| *d.file_name() == **value) {
                debug!("default {mime} in has desktop file value: {value}");
            } else {
                println!(
                    "default {mime} in has desktop file value: {value}, but this desktop-file does not exist!",
                );
            }
        }
        let mut vec = added_mimes.iter().collect::<Vec<_>>();
        vec.sort_by_key(|&(mime, _)| mime.to_string());
        for (mime, value) in vec {
            if desktop_files.iter().any(|d| *d.file_name() == **value) {
                debug!("added {mime} in has desktop file value: {value}");
            } else {
                println!(
                    "added {mime} in has desktop file value: {value}, but this desktop-file does not exist!",
                );
            }
        }
    }
}
