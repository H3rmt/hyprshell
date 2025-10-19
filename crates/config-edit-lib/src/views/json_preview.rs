use adw::gtk::{Label, Orientation};
use adw::prelude::*;
use adw::{ViewStack, gtk};

pub fn create_preview_view(view_stack: &ViewStack) {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();
    view_stack.add_titled_with_icon(&row_box, None, "Json Preview", "preview");

    let label = Label::builder()
        .label("Json preview for nix users (TODO)")
        .build();
    row_box.append(&label);
}
