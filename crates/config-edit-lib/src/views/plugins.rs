use adw::gdk::Cursor;
use adw::gtk;
use adw::gtk::{Align, Label, Orientation, Switch};
use adw::prelude::{BoxExt, WidgetExt};

pub fn create_plugins_terminal_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Run in Terminal")));
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
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_shell_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Run in Shell (background)")));
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
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_calc_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Calculator")));
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
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_path_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Open Filepath")));
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
    row.append(&hide_row);
    hide_switch
}
