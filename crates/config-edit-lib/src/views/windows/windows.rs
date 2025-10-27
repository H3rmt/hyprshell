use crate::structs::GTKWindows;
use crate::views::windows::overview::generate_overview_view;
use crate::views::windows::switch::generate_switch_view;
use adw::gdk::Cursor;
use adw::gtk::{Adjustment, Label, Orientation, SpinButton};
use adw::prelude::*;
use adw::{ExpanderRow, ViewStack, gtk};

pub fn create_windows_view(view_stack: &ViewStack) -> GTKWindows {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();
    view_stack.add_titled_with_icon(&row_box, Some("overview"), "Windows", "configure");

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(true)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .title("Windows (Overview and Switch)")
        .build();
    row_box.append(&row);

    let windows_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    row.add_row(&windows_box);

    let scale = scale(&windows_box);
    let items_per_row = items_per_row(&windows_box);

    let overview = generate_overview_view(&row);
    let switch = generate_switch_view(&row);

    GTKWindows {
        row,
        scale,
        items_per_row,
        overview,
        switch,
    }
}

fn scale(windows_box: &gtk::Box) -> SpinButton {
    let scale_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    scale_row.append(&Label::new(Some("Scale")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    scale_row.append(&info_icon);
    let scale_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.5, 15.0, 0.5, 1.0, 0.0))
        .hexpand(true)
        .digits(2)
        .build();
    scale_row.append(&scale_spin);
    windows_box.append(&scale_row);
    scale_spin
}

fn items_per_row(windows_box: &gtk::Box) -> SpinButton {
    let ipr_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    ipr_row.append(&Label::new(Some("Items per row")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    ipr_row.append(&info_icon);
    let ipr_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.0, 50.0, 1.0, 5.0, 0.0))
        .hexpand(true)
        .digits(0)
        .build();
    ipr_row.append(&ipr_spin);
    windows_box.append(&ipr_row);
    ipr_spin
}
