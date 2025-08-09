use crate::{LauncherData, plugins};
use core_lib::transfer::Identifier;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, Entry, glib};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug_span, trace, warn};

const ANIMATE_LAUNCH_MS: u64 = 500;

pub fn close_launcher_by_char(data: &mut LauncherData, char: Option<char>) {
    let _span = debug_span!("close_launcher_by_char").entered();
    if let Some(char) = char {
        data.window.set_keyboard_mode(KeyboardMode::None);
        trace!("Closing launcher with char: {}", char);
        if let Some(iden) = match char {
            '0'..='9' => char
                .to_digit(10)
                .and_then(|a| data.sorted_matches.get(a as usize)),
            _ => data.static_matches.get(&char),
        } {
            close_launcher(data, iden);
        } else {
            warn!("No match found for char: {}", char);
            close_window(&data.entry, &data.window);
        }
    } else {
        close_window(&data.entry, &data.window);
    }
}

pub fn close_launcher_by_iden(data: &mut LauncherData, iden: &Identifier) {
    let _span = debug_span!("close_launcher_by_iden").entered();
    data.window.set_keyboard_mode(KeyboardMode::None);
    trace!("Closing launcher with iden: {:?}", iden);

    close_launcher(data, iden);
}

fn close_launcher(data: &LauncherData, iden: &Identifier) {
    let span = debug_span!("close_launcher");
    let _span = span.enter();
    let instant = Instant::now();

    let animate = plugins::launch(
        iden,
        &data.entry.text(),
        data.config.default_terminal.as_deref(),
        &data.config.data_dir,
    );
    if animate {
        trace!(
            "starting timeout({}ms) animation after {:?} time",
            ANIMATE_LAUNCH_MS,
            instant.elapsed()
        );
        data.entry.set_editable(false);
        show_launch(&data.results_items, &data.plugins_items, iden);
        let window = data.window.clone();
        let entry = data.entry.clone();
        let span = span.clone();
        glib::timeout_add_local_once(Duration::from_millis(ANIMATE_LAUNCH_MS), move || {
            let _span = span.entered();
            close_window(&entry, &window);
            trace!("closed launcher after {:?} time", instant.elapsed());
        });
    } else {
        close_window(&data.entry, &data.window);
    }
}

fn close_window(entry: &Entry, window: &ApplicationWindow) {
    trace!("Hiding window (launcher) {:?}", window.id());
    window.set_visible(false);
    entry.set_text("");
}

fn show_launch(
    results_items: &HashMap<Identifier, (gtk::Box, HashMap<Identifier, gtk::ListBoxRow>)>,
    plugins_items: &HashMap<Identifier, Button>,
    open_iden: &Identifier,
) {
    for (iden, child) in results_items {
        if iden.data == open_iden.data {
            for (iden_2, row) in &child.1 {
                if iden_2.data_additional == open_iden.data_additional {
                    row.add_css_class("launch");
                    return;
                }
            }
            // only add if no child with additional data was found
            child.0.add_css_class("launch");
            return;
        }
    }
    for (iden, child) in plugins_items {
        if iden.data == open_iden.data {
            child.add_css_class("launch");
            return;
        }
    }
}
