use adw::gtk::Orientation;
use adw::prelude::*;
use adw::{ExpanderRow, gtk};

pub fn generate_launcher_view(windows_grid: &ExpanderRow) {
    let launcher_row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let label = gtk::Label::new(Some("TODO"));
    launcher_row_1.append(&label);

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(false)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .title("Launcher (TODO)")
        .build();
    row.add_row(&launcher_row_1);
    windows_grid.add_row(&row);
}
