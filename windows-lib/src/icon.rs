use crate::desktop_map::{add_path_for_icon_by_pid_exec, get_icon_name_by_name_from_desktop_files};
use core_lib::{WarnWithDetails, default};
use gtk::Image;
use std::fs;
use std::path::Path;
use tracing::{debug_span, trace, warn};

pub fn set_icon(class: &str, pid: i32, image: &Image) {
    let image = image.clone();
    let _span = debug_span!("icon", class = class).entered();

    if load_icon_from_cache(class, &image).is_some() {
        return;
    }

    if let Ok(cmdline) = fs::read_to_string(format!("/proc/{pid}/cmdline")) {
        // convert x00 to space
        trace!("No Icon found for {class}, using Icon by cmdline {cmdline} by PID ({pid})");
        let cmd = cmdline
            .split('\x00')
            .next()
            .unwrap_or_default()
            .split('/')
            .next_back()
            .unwrap_or_default()
            .split(' ')
            .next()
            .unwrap_or_default();
        if cmd.is_empty() {
            warn!("Failed to read cmdline for PID {}", pid);
        } else {
            trace!("Icon by cmdline {cmd} for {class} by PID ({pid})");
            if let Some(icon_path) = load_icon_from_cache(cmd, &image) {
                // add the icon path back into cache
                // to directly link class name to icon without checking pid again
                add_path_for_icon_by_pid_exec(class, icon_path)
                    .warn_details("Failed to add icon path to cache");
            }
        }
    } else {
        warn!("Failed to read cmdline for PID {}", pid);
    }
}

// check if the icon is in theme and apply it
fn load_icon_from_cache(name: &str, pic: &Image) -> Option<Box<Path>> {
    let name_lower = name.to_ascii_lowercase();

    if default::theme_has_icon_name(name) {
        pic.set_icon_name(Some(name));
        Some(Box::from(Path::new(name)))
    } else if default::theme_has_icon_name(&name_lower) {
        pic.set_icon_name(Some(&name_lower));
        Some(Box::from(Path::new(&name_lower)))
    } else {
        // check if icon is in desktop file cache and apply it
        if let Some((icon_path, path, source)) = get_icon_name_by_name_from_desktop_files(name) {
            trace!(
                "Found icon for {name} / {icon_path:?} in cache from source: {source:?} at {path:?}"
            );
            if icon_path.is_absolute() {
                pic.set_from_file(Some(Path::new(&*icon_path)));
            } else {
                pic.set_icon_name(icon_path.file_name().and_then(|name| name.to_str()));
            }
            Some(icon_path)
        } else {
            trace!("Icon for {name} not found in theme or cache, using `application-x-executable`");
            pic.set_icon_name(Some("application-x-executable"));
            None
        }
    }
}
