#![allow(clippy::print_stderr, clippy::print_stdout)]

use crate::plugins::get_launch_items;
use crate::reload_applications_desktop_entries_map;
use config_lib::Plugins;
use core_lib::WarnWithDetails;
use core_lib::default::reload_default_files;
use std::path::Path;
use tracing::debug;

pub fn get_matches(plugins: &Plugins, text: &str, all_items: bool, max_items: u8, data_dir: &Path) {
    reload_default_files().warn_details("Failed to reload default files");
    reload_applications_desktop_entries_map()
        .warn_details("Failed to reload applications desktop entries map");
    debug!("text: {text}");
    let options = get_launch_items(plugins, text, data_dir);
    println!("{} options returned", options.len());
    let options = if all_items {
        options
    } else {
        debug!("shorting options to {max_items}");
        options.into_iter().take(max_items as usize).collect()
    };
    for option in options {
        println!(
            "{:?}: {:?}; {} children",
            option.item.names.first(),
            option.score,
            option.item.children.len()
        );
        debug!("{option:?}");
    }
}
