use crate::structs::GTKWindows;
use crate::views::overview::generate_overview_view;
use adw::gdk::Cursor;
use adw::gtk;
use adw::gtk::{Adjustment, CheckButton, Frame, Grid, Label, Orientation, SpinButton};
use adw::prelude::*;

pub fn create_windows_view() -> (Frame, GTKWindows) {
    // Windows Section
    let label_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .css_classes(["frame-label"])
        .build();
    let activate_checkbox = CheckButton::builder().build();
    activate_checkbox.set_cursor(Cursor::from_name("pointer", None).as_ref());
    label_box.append(&activate_checkbox);
    let label = Label::builder()
        .label("Windows (Overview and Switch)")
        .build();
    label_box.append(&label);
    let windows_grid = Grid::builder()
        .orientation(Orientation::Vertical)
        .row_spacing(12)
        .column_spacing(12)
        .build();
    let windows_frame = Frame::builder()
        .label_widget(&label_box)
        .css_classes(["frame"])
        .child(&windows_grid)
        .build();

    let scale_spin = scale(&windows_grid);
    let items_per_row = items_per_row(&windows_grid);

    let _ = generate_overview_view(&windows_grid);
    //
    // let (switch_check, switch_container) = generate_switch_config(config.clone());
    // windows_box.append(&switch_check);
    // windows_box.append(&switch_container);

    (
        windows_frame,
        GTKWindows {
            enabled: activate_checkbox,
            scale: scale_spin,
            view: windows_grid,
            items_per_row,
        },
    )
}

fn scale(windows_grid: &Grid) -> SpinButton {
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
    windows_grid.attach(&scale_row, 0, 0, 1, 1);
    let scale_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.5, 15.0, 0.1, 1.0, 0.0))
        .climb_rate(0.5)
        .hexpand(true)
        .digits(2)
        .build();
    windows_grid.attach(&scale_spin, 1, 0, 1, 1);
    scale_spin
}

fn items_per_row(windows_grid: &Grid) -> SpinButton {
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
    windows_grid.attach(&ipr_row, 2, 0, 1, 1);
    let ipr_spin = SpinButton::builder()
        .adjustment(&Adjustment::new(1.0, 0.0, 50.0, 1.0, 5.0, 0.0))
        .climb_rate(1.0)
        .hexpand(true)
        .digits(0)
        .build();
    windows_grid.attach(&ipr_spin, 3, 0, 1, 1);
    ipr_spin
}
