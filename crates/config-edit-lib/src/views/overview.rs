use adw::gdk::Cursor;
use adw::gtk::{
    Align, CheckButton, DropDown, Entry, Frame, Grid, InputPurpose, Label, Orientation, Switch,
};
use adw::prelude::*;
use adw::{ExpanderRow, SwitchRow, gtk};

pub fn generate_overview_view(windows_grid: &gtk::Grid) {
    let label_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .css_classes(["frame-label"])
        .build();
    let activate_checkbox = CheckButton::builder().build();
    activate_checkbox.set_cursor(Cursor::from_name("pointer", None).as_ref());
    label_box.append(&activate_checkbox);
    let label = Label::builder().label("Overview").build();
    label_box.append(&label);
    let overview_grid = Grid::builder()
        .orientation(Orientation::Vertical)
        .row_spacing(12)
        .column_spacing(12)
        .build();
    let windows_frame = Frame::builder()
        .label_widget(&label_box)
        .css_classes(["frame"])
        .hexpand(true)
        .child(&overview_grid)
        .build();
    windows_grid.attach(&windows_frame, 0, 2, 4, 1);

    let key = key(&overview_grid);
    let modifier = modifier(&overview_grid);
    let filter = filter(&overview_grid);
    let hide_filtered = hide_filtered(&overview_grid);
}

fn key(windows_grid: &Grid) -> Entry {
    let key_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    key_row.append(&Label::new(Some("Key")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    key_row.append(&info_icon);
    windows_grid.attach(&key_row, 0, 0, 1, 1);
    let key_entry = Entry::builder()
        .input_purpose(InputPurpose::FreeForm)
        .placeholder_text("super_l")
        .css_classes(["entry"])
        .hexpand(true)
        .build();
    windows_grid.attach(&key_entry, 1, 0, 1, 1);
    key_entry
}

fn modifier(windows_grid: &Grid) -> DropDown {
    let mod_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    mod_row.append(&Label::new(Some("Modifier")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    mod_row.append(&info_icon);
    windows_grid.attach(&mod_row, 2, 0, 1, 1);
    let dropdown = DropDown::from_strings(&["Alt", "Ctrl", "Super"]);
    dropdown.set_hexpand(true);
    windows_grid.attach(&dropdown, 3, 0, 1, 1);
    dropdown
}

fn filter(windows_grid: &Grid) {
    let filter_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    filter_row.append(&Label::new(Some("Filter")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    filter_row.append(&info_icon);
    windows_grid.attach(&filter_row, 0, 2, 1, 1);

    let expander = ExpanderRow::builder().title("Filter").build();
    let sw_same = SwitchRow::new();
    sw_same.set_title("Same class");
    expander.add_row(&sw_same);
    let sw_workspace = SwitchRow::new();
    sw_workspace.set_title("Current workspace");
    expander.add_row(&sw_workspace);
    let sw_monitor = SwitchRow::new();
    sw_monitor.set_title("Current monitor");
    expander.add_row(&sw_monitor);

    windows_grid.attach(&expander, 1, 2, 1, 1);
}

fn hide_filtered(windows_grid: &Grid) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["frame-row"])
        .build();
    hide_row.append(&Label::new(Some("Hide filtered")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    windows_grid.attach(&hide_row, 2, 2, 1, 1);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    switch_box.append(&hide_switch);
    windows_grid.attach(&switch_box, 3, 2, 1, 1);
    hide_switch
}
