use adw::gtk::{Button, DropDown, Entry, SpinButton, Switch};
use adw::{ExpanderRow, SwitchRow};

pub struct GTKConfig {
    pub windows: GTKWindows,
    pub save: Button,
}

pub struct GTKWindows {
    pub row: ExpanderRow,
    pub scale: SpinButton,
    pub items_per_row: SpinButton,
    pub overview: GTKOverview,
    pub switch: GTKSwitch,
}

pub struct GTKOverview {
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

pub struct GTKSwitch {
    pub row: ExpanderRow,
    pub modifier: DropDown,
    pub filter: GTKWindowsFilter,
    pub switch_workspaces: Switch,
}
