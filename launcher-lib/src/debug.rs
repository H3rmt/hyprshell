use crate::plugins::get_sortable_launch_options;
use crate::reload_applications_desktop_entries_map;
use config_lib::Plugins;
use core_lib::default::reload_default_files;
use std::path::Path;
use tracing::{debug, info};

pub fn get_matches(plugins: &Plugins, text: &str, all_items: bool, max_items: u8, data_dir: &Path) {
    reload_default_files();
    reload_applications_desktop_entries_map();
    debug!("text: {text}");
    let options = get_sortable_launch_options(plugins, text, data_dir);
    info!("{} options returned", options.len());
    let options = if all_items {
        options
    } else {
        debug!("shorting options to {max_items}");
        options.into_iter().take(max_items as usize).collect()
    };
    for option in options {
        info!(
            "{}: {}; {} desktop actions",
            option.name,
            option.score,
            option.details_menu.len()
        );
        debug!("{option:?}");
    }
}
