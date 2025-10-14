use crate::APPLICATION_EDIT_ID;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};
use gtk::{Application, ApplicationWindow};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

#[instrument]
pub fn start(config_path: PathBuf, css_path: PathBuf) {
    // Create a new application
    let application = Application::builder()
        .application_id(format!(
            "{}{}",
            APPLICATION_EDIT_ID,
            if cfg!(debug_assertions) { "-test" } else { "" }
        ))
        .build();
    debug!("Application created");

    application.connect_activate(move |app| {
        activate(app, &config_path, &css_path);
    });
    let exit = application.run_with_args::<String>(&[]);
    debug!("Application exited with code {exit:?}");
}

fn activate(app: &Application, config_path: &Path, css_path: &Path) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprshell Config Editor")
        .resizable(false) // TODO make resizable
        .default_width(800)
        .default_height(600)
        .build();

    create_config_view(&window);

    // Present window
    window.present();
}

fn create_config_view(window: &ApplicationWindow) {
    // config_lib::load_and_migrate_config();
    // config_lib::write_config()
}
