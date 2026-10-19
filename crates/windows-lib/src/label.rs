use crate::desktop_map::get_app_name_by_class;
use crate::icon::set_icon;
use core_lib::ClientData;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::gtk::pango;

const LABEL_ICON_SIZE: i32 = 18;

/// Card header: app icon, app name and window title, so a window can be told apart even when
/// its title or thumbnail isn't helpful.
pub fn client_label(data: &ClientData) -> gtk::Box {
    let label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    label.set_css_classes(&["client-label"]);
    label.set_halign(gtk::Align::Center);
    label.set_overflow(gtk::Overflow::Visible);
    label.set_margin_start(6);
    label.set_margin_end(6);

    let icon = gtk::Image::new();
    icon.set_css_classes(if data.enabled {
        &["client-label-icon"]
    } else {
        &["client-label-icon", "monochrome"]
    });
    icon.set_pixel_size(LABEL_ICON_SIZE);
    set_icon(&data.class, data.pid, &icon);
    label.append(&icon);

    let title = data.title.trim();
    let app_name =
        get_app_name_by_class(&data.class).map_or_else(|| data.class.clone(), String::from);
    let app = gtk::Label::new(Some(&app_name));
    app.set_css_classes(&["client-app-name"]);
    app.set_ellipsize(pango::EllipsizeMode::End);
    label.append(&app);

    // the class is already shown as the app name, don't repeat it for untitled windows
    if !title.is_empty() {
        let title = gtk::Label::new(Some(title));
        title.set_css_classes(&["client-title"]);
        title.set_ellipsize(pango::EllipsizeMode::End);
        label.append(&title);
    }
    label
}
