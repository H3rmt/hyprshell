use crate::LauncherData;
use adw::gtk::glib;
use adw::gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use tracing::{debug_span, trace};

pub fn open_launcher(data: &LauncherData) {
    let _span = debug_span!("open_launcher").entered();
    // check if already open
    if data.window.get_visible() {
        return;
    }

    trace!("Showing window {:?}", data.window.id());
    data.window.set_monitor(None);
    data.window.set_visible(true);

    trace!("Resetting launcher data");
    data.entry.set_editable(true);
    data.entry.set_text("");
    data.window.set_keyboard_mode(KeyboardMode::Exclusive);
    let window = data.window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        glib::ControlFlow::Break
    });

    data.window.grab_focus();
    data.entry.grab_focus();
}
