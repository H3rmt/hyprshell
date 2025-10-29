use crate::APPLICATION_EDIT_ID;
use crate::bind::bind;
use crate::footer::footer;
use crate::structs::GTKConfig;
use crate::update::update_config;
use crate::update_changes_view::set_previous_config;
use crate::views::changes::create_changes_view;
use crate::views::json_preview::create_preview_view;
use crate::views::windows::windows::create_windows_view;
use adw::gdk::Display;
use adw::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, ScrolledWindow,
    style_context_add_provider_for_display,
};
use adw::prelude::*;
use adw::{
    AlertDialog, Application, ApplicationWindow, ToolbarStyle, ToolbarView, ViewStack,
    ViewSwitcherBar, glib,
};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument, warn};

#[instrument]
pub fn start(config_path: PathBuf, css_path: PathBuf) {
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
    set_previous_config(config.clone());

    let view_stack = ViewStack::builder().build();
    create_preview_view(&view_stack);
    let (changes, how_to_use) = create_changes_view(&view_stack);
    let windows = create_windows_view(&view_stack);
    view_stack.set_visible_child_name("overview");

    let scroll = ScrolledWindow::builder().child(&view_stack).build();
    let view = ToolbarView::builder()
        .top_bar_style(ToolbarStyle::Raised)
        .bottom_bar_style(ToolbarStyle::Flat)
        .reveal_bottom_bars(true)
        .reveal_top_bars(true)
        .content(&scroll)
        .build();

    let switcher = ViewSwitcherBar::builder()
        .reveal(true)
        .stack(&view_stack)
        .build();
    let header = adw::HeaderBar::builder()
        .show_end_title_buttons(true)
        .show_start_title_buttons(true)
        .show_back_button(true) // TODO find out why this is not working
        .title_widget(&switcher)
        .build();
    view.add_top_bar(&header);

    let (footer, save) = footer(&window, config_path);
    view.add_bottom_bar(&footer);

    window.set_content(Some(&view));

    let gtk_config = GTKConfig {
        changes,
        how_to_use,
        windows,
        save,
        view_stack,
        path: PathBuf::from(config_path).into_boxed_path(),
    };
    update_config(&gtk_config, &config);
    bind(gtk_config, config);

    window.present();
}
