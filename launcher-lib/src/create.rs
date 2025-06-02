use crate::global::LauncherGlobalData;
use crate::{LauncherGlobal, update_launcher};
use core_lib::transfer::TransferType;
use core_lib::{LAUNCHER_NAMESPACE, Warn, send_to_socket};
use gtk::Orientation;
use gtk::gdk::Key;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, EventControllerKey, ListBox, SelectionMode};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::collections::HashMap;
use tracing::{Level, debug, span};

pub fn create_launcher_window(
    app: &Application,
    global: &mut LauncherGlobal,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "create_launcher_window").entered();

    let main_vbox = ListBox::builder()
        .css_classes(["launcher"])
        .width_request(global.width as i32)
        .selection_mode(SelectionMode::None)
        .build();

    let entry = Entry::builder().css_classes(["launcher-input"]).build();
    entry.connect_changed(|e| {
        // trace!("Launcher entry changed: {}", e.text());
        send_to_socket(&TransferType::Type(e.text().to_string()))
            .warn("unable send return to socket");
    });
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, k, _, _| match k {
        Key::Tab => Propagation::Stop,
        Key::ISO_Left_Tab => Propagation::Stop,
        _ => Propagation::Proceed,
    });
    entry.add_controller(key_controller);
    main_vbox.append(&entry);

    let results = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(["launcher-results"])
        .build();
    main_vbox.append(&results);

    let plugin_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["launcher-plugins"])
        .spacing(5)
        .hexpand(false)
        .vexpand(false)
        .build();
    main_vbox.append(&plugin_box);

    let window = ApplicationWindow::builder()
        .css_classes(["window"])
        .application(app)
        .child(&main_vbox)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_namespace(Some(LAUNCHER_NAMESPACE));
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 15);
    window.present();
    window.set_visible(false);

    debug!("Created launcher window ({})", window.id());
    global.data = Some(RefCell::new(LauncherGlobalData {
        window,
        entry,
        results,
        plugin_box,
        sorted_matches: vec![],
        static_matches: HashMap::new(),
    }));

    // initial update
    update_launcher(global, "".to_string());

    Ok(())
}
