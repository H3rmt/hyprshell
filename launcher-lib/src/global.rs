use config_lib::{Modifier, Plugins};
use core_lib::transfer::Identifier;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct LauncherData {
    pub config: LauncherConfig,
    pub window: gtk::ApplicationWindow,
    pub entry: gtk::Entry,
    pub results_box: gtk::Box,
    pub results_items: HashMap<Identifier, (gtk::Box, HashMap<Identifier, gtk::ListBoxRow>)>,
    pub plugins_box: gtk::Box,
    pub plugins_items: HashMap<Identifier, gtk::Button>,
    pub sorted_matches: Vec<Identifier>,
    pub static_matches: HashMap<char, Identifier>,
}

#[derive(Debug)]
pub struct LauncherConfig {
    pub default_terminal: Option<Box<str>>,
    pub max_items: u8,
    pub launch_modifier: Modifier,
    pub show_when_empty: bool,
    pub width: u32,
    pub data_dir: Box<Path>,
    pub plugins: Plugins,
}
