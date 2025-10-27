use crate::structs::{GTKOverview, GTKWindowsFilter};
use crate::views::launcher::launcher::create_launcher_view;
use adw::gdk::Cursor;
use adw::gtk::{Align, DropDown, Entry, InputPurpose, Label, Orientation, Switch};
use adw::prelude::*;
use adw::{ExpanderRow, SwitchRow, gtk};

pub fn generate_overview_view(windows_grid: &ExpanderRow) -> GTKOverview {
    let overview_row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let key = key(&overview_row_1);
    let modifier = modifier(&overview_row_1);
    let overview_row_2 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let filter = filter(&overview_row_2);
    let hide_filtered = hide_filtered(&overview_row_2);

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(true)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .title("Overview + Launcher")
        .build();
    row.add_row(&overview_row_1);
    row.add_row(&overview_row_2);
    windows_grid.add_row(&row);

    let launcher = create_launcher_view();

    GTKOverview {
        launcher,
        row,
        key,
        modifier,
        filter,
        hide_filtered,
    }
}

fn key(windows_box: &gtk::Box) -> Entry {
    let key_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    key_row.append(&Label::new(Some("Key")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    key_row.append(&info_icon);
    let key_entry = Entry::builder()
        .input_purpose(InputPurpose::FreeForm)
        .placeholder_text("super_l")
        .hexpand(true)
        .build();
    key_row.append(&key_entry);
    windows_box.append(&key_row);
    key_entry
}

fn modifier(windows_box: &gtk::Box) -> DropDown {
    let mod_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    mod_row.append(&Label::new(Some("Modifier")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    mod_row.append(&info_icon);
    // DO NOT CHANGE ORDER OF THESE ITEMS
    let dropdown = DropDown::from_strings(&["Alt", "Ctrl", "Super"]);
    dropdown.set_hexpand(true);
    mod_row.append(&dropdown);
    windows_box.append(&mod_row);
    dropdown
}

fn filter(windows_box: &gtk::Box) -> GTKWindowsFilter {
    let filter_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    filter_row.append(&Label::new(Some("Filter")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    filter_row.append(&info_icon);

    let expander = ExpanderRow::builder()
        .title("Filter")
        .hexpand(true)
        .title_lines(2)
        .css_classes(["item-expander"])
        .build();
    let sw_same = SwitchRow::new();
    sw_same.set_title("Same class");
    expander.add_row(&sw_same);
    let sw_workspace = SwitchRow::new();
    sw_workspace.set_title("Current workspace");
    expander.add_row(&sw_workspace);
    let sw_monitor = SwitchRow::new();
    sw_monitor.set_title("Current monitor");
    expander.add_row(&sw_monitor);

    filter_row.append(&expander);
    windows_box.append(&filter_row);
    GTKWindowsFilter {
        row: expander,
        same_class: sw_same,
        workspace: sw_workspace,
        monitor: sw_monitor,
    }
}

fn hide_filtered(windows_box: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Hide filtered")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    switch_box.append(&hide_switch);
    hide_row.append(&switch_box);
    windows_box.append(&hide_row);
    hide_switch
}
