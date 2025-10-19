use adw::gdk::Cursor;
use adw::gtk::{DropDown, Label, Orientation};
use adw::prelude::*;
use adw::{ExpanderRow, ViewStack, gtk};

pub fn create_launcher_view(view_stack: &ViewStack) {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();
    view_stack.add_titled_with_icon(&row_box, None, "Launcher", "configure");

    let row = ExpanderRow::builder()
        .title_selectable(true)
        .show_enable_switch(false)
        .hexpand(true)
        .expanded(true)
        .css_classes(["enable-frame"])
        .title("Launcher (TODO)")
        .build();
    row_box.append(&row);

    let launcher_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    let _modifier = launch_modifier(&launcher_box);

    row.add_row(&launcher_box);
}

fn launch_modifier(windows_box: &gtk::Box) -> DropDown {
    let mod_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    mod_row.append(&Label::new(Some("Modifier")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("Adjust the scale factor for window previews."));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    mod_row.append(&info_icon);
    // DO NOT CHANGE ORDER OF THESE ITEMS
    let dropdown = DropDown::from_strings(&["Alt", "Ctrl", "Super"]);
    dropdown.set_hexpand(true);
    mod_row.append(&dropdown);
    windows_box.append(&mod_row);
    dropdown
}
