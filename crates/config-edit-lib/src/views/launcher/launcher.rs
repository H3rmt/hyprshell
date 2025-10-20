use crate::structs::GTKLauncher;
use crate::views::launcher::plugins::plugins_rows;
use adw::gdk::Cursor;
use adw::gtk::{
    Adjustment, Align, DropDown, Entry, InputPurpose, Label, Orientation, SpinButton, Switch,
};
use adw::prelude::*;
use adw::{ExpanderRow, gtk};

pub fn create_launcher_view() -> GTKLauncher {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(false)
        .hexpand(true)
        .expanded(true)
        .css_classes(["enable-frame"])
        .title("Launcher")
        .build();
    row_box.append(&row);

    let launcher_box_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let modifier = launch_modifier(&launcher_box_1);
    let width = width(&launcher_box_1);
    let max_items = max_items(&launcher_box_1);

    let launcher_box_2 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let (dont_use_default_terminal, terminal) = terminal(&launcher_box_2);
    let show_when_empty = show_when_empty(&launcher_box_2);

    let plugins_row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(false)
        .hexpand(true)
        .expanded(true)
        .css_classes(["enable-frame"])
        .title("Plugins")
        .build();

    let plugins = plugins_rows(&plugins_row);

    row.add_row(&launcher_box_1);
    row.add_row(&launcher_box_2);
    row.add_row(&plugins_row);

    GTKLauncher {
        view: row_box,
        row,
        modifier,
        dont_use_default_terminal,
        terminal,
        width,
        max_items,
        show_when_empty,
        plugins,
    }
}

fn launch_modifier(windows_box: &gtk::Box) -> DropDown {
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

fn width(windows_box: &gtk::Box) -> SpinButton {
    let width = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    width.append(&Label::new(Some("Width")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    width.append(&info_icon);
    let ipr_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(0.0, 0.0, 2000.0, 50.0, 100.0, 0.0))
        .hexpand(true)
        .digits(0)
        .build();
    width.append(&ipr_spin);
    windows_box.append(&width);
    ipr_spin
}

fn max_items(windows_box: &gtk::Box) -> SpinButton {
    let max_items = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    max_items.append(&Label::new(Some("Max items")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    max_items.append(&info_icon);
    let ipr_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(0.0, 0.0, 20.0, 1.0, 2.0, 0.0))
        .hexpand(true)
        .digits(0)
        .build();
    max_items.append(&ipr_spin);
    windows_box.append(&max_items);
    ipr_spin
}

fn show_when_empty(windows_box: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Show when empty")));
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

fn terminal(windows_box: &gtk::Box) -> (Switch, Entry) {
    let key_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    key_row.append(&Label::new(Some("Default terminal")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    key_row.append(&info_icon);
    let dont_use_default_terminal = Switch::builder().valign(Align::Center).build();
    key_row.append(&dont_use_default_terminal);
    let key_entry = Entry::builder()
        .input_purpose(InputPurpose::FreeForm)
        .placeholder_text("kitty")
        .hexpand(true)
        .build();
    key_row.append(&key_entry);
    windows_box.append(&key_row);
    (dont_use_default_terminal, key_entry)
}
