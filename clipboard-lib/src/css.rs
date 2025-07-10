use gtk::gdk::Display;
use gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, glib, style_context_add_provider_for_display,
};
pub fn get_css() {
    let provider_app = CssProvider::new();
    provider_app.load_from_bytes(&glib::Bytes::from_static(include_bytes!("styles.css")));
    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_USER,
    );
}
