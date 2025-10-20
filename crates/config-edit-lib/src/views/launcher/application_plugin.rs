use adw::gdk::Cursor;
use adw::gtk;
use adw::gtk::{Adjustment, Align, Label, Orientation, SpinButton, Switch};
use adw::prelude::*;

pub fn show_execs(windows_box: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Show execs")));
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

pub fn application_submenu(windows_box: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Show Actions")));
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

pub fn run_cache_weeks(windows_box: &gtk::Box) -> SpinButton {
    let gbox = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    gbox.append(&Label::new(Some("Run Cache period (weeks)")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    gbox.append(&info_icon);
    let spin = SpinButton::builder()
        .adjustment(&Adjustment::new(0.0, 0.0, 52.0, 1.0, 8.0, 0.0))
        .hexpand(true)
        .digits(0)
        .build();
    gbox.append(&spin);
    windows_box.append(&gbox);
    spin
}
