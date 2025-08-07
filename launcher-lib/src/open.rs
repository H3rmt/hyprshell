use crate::LauncherData;
use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use tracing::{Level, debug_span, span, trace};

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
    glib::idle_add_local(move || {
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        glib::ControlFlow::Break
    });

    data.window.grab_focus();
    data.entry.grab_focus();
}
