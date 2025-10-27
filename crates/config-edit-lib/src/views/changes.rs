use adw::gtk::{Align, ListBox, Orientation, SelectionMode, TextView};
use adw::prelude::*;
use adw::{ViewStack, gtk};

pub fn create_changes_view(view_stack: &ViewStack) -> (ListBox, TextView) {
    let row_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .build();
    view_stack.add_titled_with_icon(&row_box, None, "Changes", "document-edit-symbolic");

    let list = ListBox::builder()
        .css_classes(["changes-list"])
        .selection_mode(SelectionMode::None)
        .show_separators(false)
        .halign(Align::Center)
        .valign(Align::Start)
        .hexpand(true)
        .build();
    row_box.append(&list);

    let text = TextView::builder()
        .css_classes(["changes-text"])
        .editable(false)
        .sensitive(false)
        .valign(Align::Fill)
        .halign(Align::Fill)
        .build();
    row_box.append(&text);

    (list, text)
}
