use crate::APPLICATION_EDIT_ID;
use crate::components::root::{Root, RootInit};
use relm4::RelmApp;
use relm4::adw::gdk::Display;
use relm4::adw::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, style_context_add_provider_for_display,
};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// # Panics
/// if no display was found
pub fn start(
    config_file: PathBuf,
    css_path: PathBuf,
    system_data_dirs: Vec<PathBuf>,
    generate: bool,
) {
    let config_file: Rc<Path> = Rc::from(config_file.as_path());
    let css_path: Rc<Path> = Rc::from(css_path.as_path());
    let system_data_dirs: Rc<[Box<Path>]> = Rc::from(
        system_data_dirs
            .into_iter()
            .map(|d| d.into_boxed_path())
            .collect::<Vec<_>>(),
    );

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
        config_file,
        css_path,
        system_data_dirs,
        generate,
    });
}
