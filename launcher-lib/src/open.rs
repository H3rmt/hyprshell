use crate::LauncherGlobal;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use tracing::{Level, span, trace};

pub fn open_launcher(global: &LauncherGlobal) {
    let _span = span!(Level::TRACE, "open_launcher").entered();

    if let Some(data) = &global.data {
        let data = data.borrow();
        trace!("Showing window {:?}", data.window.id());
        data.window.set_monitor(None);
        data.window.set_visible(true);

        trace!("Resetting launcher data");
        data.entry.set_editable(true);
        data.entry.set_text("");

        data.window.set_keyboard_mode(KeyboardMode::Exclusive);
        data.window.grab_focus();
        data.entry.grab_focus();
    }
}
