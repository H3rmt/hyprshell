use crate::APPLICATION_EDIT_ID;
use crate::components::root::{Root, RootInit};
use relm4::RelmApp;
use relm4::adw::gdk::Display;
use relm4::adw::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, style_context_add_provider_for_display,
};
use std::path::PathBuf;

/// # Panics
/// if no display was found
pub fn start(config_file: PathBuf, css_file: PathBuf, system_data_dir: PathBuf, generate: bool) {
    let relm = RelmApp::new(&format!(
        "{}{}",
        APPLICATION_EDIT_ID,
        if cfg!(debug_assertions) { "-test" } else { "" }
    ))
    .with_args(vec![]);

    let provider_app = CssProvider::new();
    provider_app.load_from_string(include_str!("styles.css"));
    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    relm.run::<Root>(RootInit {
        config_file: config_file.into_boxed_path(),
        system_data_dir: system_data_dir.into_boxed_path(),
        css_file: css_file.into_boxed_path(),
        generate,
    });
}
