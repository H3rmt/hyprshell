use adw::gtk::{Button, DropDown, Entry, ListBox, SpinButton, Switch, TextView};
use adw::{ExpanderRow, SwitchRow, ViewStack, gtk};
use std::path::Path;

pub struct GTKConfig {
    pub windows: GTKWindows,
    pub save: Button,
    pub view_stack: ViewStack,
    pub changes: ListBox,
    pub how_to_use: TextView,
    pub path: Box<Path>,
}

pub struct GTKWindows {
    pub row: ExpanderRow,
    pub scale: SpinButton,
    pub items_per_row: SpinButton,
    pub overview: GTKOverview,
    pub switch: GTKSwitch,
}

pub struct GTKOverview {
    pub launcher: GTKLauncher,
    pub row: ExpanderRow,
    pub key: Entry,
    pub modifier: DropDown,
    pub filter: GTKWindowsFilter,
    pub hide_filtered: Switch,
}

pub struct GTKWindowsFilter {
    pub row: ExpanderRow,
    pub same_class: SwitchRow,
    pub workspace: SwitchRow,
    pub monitor: SwitchRow,
}

pub struct GTKLauncher {
    pub view: gtk::Box,
    pub row: ExpanderRow,
    pub modifier: DropDown,
    pub width: SpinButton,
    pub max_items: SpinButton,
    pub show_when_empty: Switch,
    pub dont_use_default_terminal: Switch,
    pub terminal: Entry,
    pub plugins: GTKPlugins,
}

pub struct GTKPlugins {
    pub row: ExpanderRow,
    pub terminal: Switch,
    pub shell: Switch,
    pub calc: Switch,
    pub path: Switch,
    pub applications: GTKApplications,
}

pub struct GTKApplications {
    pub row: ExpanderRow,
    pub cache_weeks: SpinButton,
    pub submenu: Switch,
    pub show_exec: Switch,
}

pub struct GTKSwitch {
    pub row: ExpanderRow,
    pub modifier: DropDown,
    pub filter: GTKWindowsFilter,
    pub switch_workspaces: Switch,
}
