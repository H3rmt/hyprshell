use crate::structs::GTKWindows;
use crate::views::overview::generate_overview_view;
use adw::gdk::Cursor;
use adw::gtk::{Adjustment, Label, Orientation, SpinButton};
use adw::prelude::*;
use adw::{ExpanderRow, gtk};

pub fn create_windows_view(settings: &gtk::Box) -> GTKWindows {
    let windows_grid = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-grid"])
        .spacing(12)
        .build();

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(true)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .title("Windows (Overview and Switch)")
        .build();
    row.add_row(&windows_grid);
    settings.append(&row);

    let scale = scale(&windows_grid);
    let items_per_row = items_per_row(&windows_grid);

    let overview = generate_overview_view(&row);

    GTKWindows {
        row,
        scale,
        items_per_row,
        overview,
    }
}

fn scale(windows_box: &gtk::Box) -> SpinButton {
    let scale_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    scale_row.append(&Label::new(Some("Scale")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    scale_row.append(&info_icon);
    windows_box.append(&scale_row);
    let scale_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.5, 15.0, 0.1, 1.0, 0.0))
        .climb_rate(0.5)
        .hexpand(true)
        .digits(2)
        .build();
    windows_box.append(&scale_spin);
    scale_spin
}

fn items_per_row(windows_box: &gtk::Box) -> SpinButton {
    let ipr_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    ipr_row.append(&Label::new(Some("Items per row")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the number of items per row in the overview."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    ipr_row.append(&info_icon);
    windows_box.append(&ipr_row);
    let ipr_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.0, 50.0, 1.0, 5.0, 0.0))
        .climb_rate(1.0)
        .hexpand(true)
        .digits(0)
        .build();
    windows_box.append(&ipr_spin);
    ipr_spin
}
