use adw::gtk::Orientation;
use adw::prelude::*;
use adw::{ViewStack, gtk};

pub fn create_changes_view(view_stack: &ViewStack) -> gtk::Box {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();
    view_stack.add_titled_with_icon(&row_box, None, "Changes", "document-edit-symbolic");

    let b = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    row_box.append(&b);

    b
}
