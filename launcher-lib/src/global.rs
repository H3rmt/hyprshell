use core_lib::config::{Launcher, Plugins};
use core_lib::transfer::Identifier;
use gtk::{ApplicationWindow, Entry, ListBox};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct LauncherGlobal {
    pub default_terminal: Option<Box<str>>,
    pub max_items: u8,
    pub show_when_empty: bool,
    pub animate_launch_ms: u64,
    pub width: u32,
    pub data_dir: Box<Path>,
    pub plugins: Plugins,
    pub data: Option<RefCell<LauncherGlobalData>>,
}

impl LauncherGlobal {
    pub fn new(data_dir: &Path) -> Box<dyn FnOnce(Launcher) -> LauncherGlobal> {
        let data_dir = Box::from(data_dir);
        Box::from(move |config: Launcher| Self {
            default_terminal: config.default_terminal.clone(),
            max_items: config.max_items,
            show_when_empty: config.show_when_empty,
            animate_launch_ms: config.animate_launch_ms,
            width: config.width,
            data_dir,
            plugins: config.plugins.clone(),
            data: None,
        })
    }
}

#[derive(Debug)]
pub struct LauncherGlobalData {
    pub window: ApplicationWindow,
    pub entry: Entry,
    pub results: ListBox,
    pub plugin_box: gtk::Box,
    pub sorted_matches: Vec<Identifier>,
    pub static_matches: HashMap<char, Identifier>,
}
