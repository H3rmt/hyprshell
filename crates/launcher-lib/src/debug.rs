#![allow(clippy::print_stderr, clippy::print_stdout)]

use crate::plugin::{MatchedLaunchItem, match_launch_item};
use crate::plugins::get_static_items;
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
    let results = get_static_items(plugins, data_dir);
    let mut results: Vec<MatchedLaunchItem> = results
        .into_iter()
        .filter_map(|item| match_launch_item(item, text))
        .collect();
    // reverse sorting, so that the most relevant items are at the top
    results.sort_by_key(|b| std::cmp::Reverse(b.score));
    println!("{} options returned", results.len());
    let options = if all_items {
        results
    } else {
        debug!("shorting options to {max_items}");
        results.into_iter().take(max_items as usize).collect()
    };
    for option in options {
        println!(
            "{}: {:?}; {} children. bonus: {}, highlight: {:?}",
            option.item.name,
            option.score,
            option.item.children.len(),
            option.item.bonus_score,
            option.highlight,
        );
        debug!("{option:?}");
    }
}
