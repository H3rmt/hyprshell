use crate::APPLICATION_EDIT_ID;
use crate::bind::bind;
use crate::footer::footer;
use crate::structs::GTKConfig;
use crate::update::update_config;
use crate::views::windows::create_windows_view;
use adw::gdk::Display;
use adw::gtk::{
    CssProvider, Orientation, STYLE_PROVIDER_PRIORITY_APPLICATION, ScrolledWindow,
    style_context_add_provider_for_display,
};
use adw::prelude::*;
use adw::{AlertDialog, Application, ApplicationWindow, ToolbarStyle, ToolbarView, glib, gtk};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument, warn};

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

fn activate(app: &Application, config_path: &Path, _css_path: &Path) {
    let provider_app = CssProvider::new();
    provider_app.load_from_bytes(&glib::Bytes::from_static(include_bytes!("styles.css")));
    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprshell Config Editor")
        .resizable(true)
        .default_width(900)
        .default_height(700)
        .build();

    let config = match config_lib::load_and_migrate_config(config_path, true) {
        Ok(c) => c,
        Err(err) => {
            warn!("Failed to load config: {err:?}");
            let dialog = AlertDialog::builder()
                .heading("Failed to load config")
                .body(format!("{err:#}"))
                .close_response("close")
                .build();
            dialog.add_responses(&[("close", "Close")]);
            window.present();
            let app = app.clone();
            glib::spawn_future_local(async move {
                let res = dialog.choose_future(&window).await;
                debug!("Dialog closed: {res:?}");
                app.quit();
            });
            return;
        }
    };

    let settings = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(12)
        .margin_top(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let root = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(12)
        .margin_top(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&settings)
        .build();
    root.append(&scroll);

    let windows = create_windows_view(&settings);
    let (footer, save) = footer(&window, config_path);
    // root.append(&footer);

    let header = adw::HeaderBar::builder()
        // .show_title_buttons(true)
        // .use_native_controls(true)
        .build();

    let view = ToolbarView::builder()
        .top_bar_style(ToolbarStyle::Raised)
        .bottom_bar_style(ToolbarStyle::Raised)
        .extend_content_to_bottom_edge(true)
        .reveal_bottom_bars(true)
        .reveal_top_bars(true)
        .content(&root)
        .build();
    view.add_top_bar(&header);
    view.add_bottom_bar(&footer);
    window.set_content(Some(&view));

    let gtk_config = GTKConfig { windows, save };
    update_config(&gtk_config, &config);
    bind(gtk_config, config);

    window.present();
}
