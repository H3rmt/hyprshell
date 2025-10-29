use adw::gdk::Cursor;
use adw::gtk::{ActionBar, Button, Orientation};
use adw::prelude::*;
use adw::{ApplicationWindow, gtk};
use std::path::Path;

pub fn footer(window: &ApplicationWindow, config_path: &Path) -> (ActionBar, Button) {
    let footer = gtk::Box::builder()
        .spacing(20)
        .hexpand(true)
        .css_classes(["footer"])
        .orientation(Orientation::Horizontal)
        .build();
    let bar = ActionBar::builder().build();
    bar.set_center_widget(Some(&footer));

    let version_label = gtk::Label::builder()
        .label(format!("Hyprshell v{}", env!("CARGO_PKG_VERSION")))
        .build();
    footer.append(&version_label);

    let buttons = gtk::Box::builder()
        .spacing(10)
        .halign(gtk::Align::End)
        .hexpand(true)
        .orientation(Orientation::Horizontal)
        .build();
    footer.append(&buttons);
    let save = Button::builder()
        .label("Save Changes")
        .css_classes(["suggested-action"])
        .tooltip_text(format!("Config file: {}", config_path.display()))
        .build();
    save.set_cursor(Cursor::from_name("pointer", None).as_ref());
    buttons.append(&save);
    let cancel = Button::builder()
        .label("Cancel")
        .css_classes(["destructive-action"])
        .build();
    cancel.set_cursor(Cursor::from_name("pointer", None).as_ref());
    buttons.append(&cancel);

    let window = window.clone();
    cancel.connect_clicked(move |_| {
        window.close();
    });
    (bar, save)
}
