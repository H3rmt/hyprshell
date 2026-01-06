use anyhow::Context;
use relm4::adw::gtk::gdk::Display;
use relm4::adw::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, style_context_add_provider_for_display,
};

pub fn get_css() -> anyhow::Result<()> {
    let provider_app = CssProvider::new();
    provider_app.load_from_string(include_str!("styles.css"));
    style_context_add_provider_for_display(
        &Display::default().context("Could not connect to a display.")?,
        &provider_app,
        STYLE_PROVIDER_PRIORITY_USER,
    );
    Ok(())
}
