use crate::global::{LauncherConfig, LauncherData};
use crate::plugins::get_static_options_chars;
use async_channel::Sender;
use core_lib::config::{Launcher, Mod};
use core_lib::transfer::{CloseOverviewConfig, Direction, SwitchOverviewConfig, TransferType};
use core_lib::{LAUNCHER_NAMESPACE, WarnWithDetails};
use gtk::gdk::Key;
use gtk::glib::{ControlFlow, Propagation};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Entry, EventControllerKey, ListBox, PropagationPhase,
    SelectionMode,
};
use gtk::{Orientation, glib};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{Level, debug, span};

pub fn create_windows_overview_launcher_window(
    app: &Application,
    launcher: &Launcher,
    open_modifier: Mod,
    data_dir: &Path,
    event_sender: Sender<TransferType>,
) -> anyhow::Result<LauncherData> {
    let _span = span!(Level::TRACE, "create_windows_overview_launcher_window").entered();

    let main_vbox = ListBox::builder()
        .css_classes(["launcher"])
        .width_request(launcher.width as i32)
        .selection_mode(SelectionMode::None)
        .build();

    let entry = Entry::builder().css_classes(["launcher-input"]).build();
    let event_sender_2 = event_sender.clone();
    entry.connect_changed(move |e| {
        launcher_entry_text_change(e.text().to_string(), event_sender_2.clone());
    });
    let event_controller = EventControllerKey::new();
    let plugin_keys = get_static_options_chars(&launcher.plugins);
    let event_sender_3 = event_sender.clone();
    let modifiers = Arc::new(Mutex::new(0u16));
    let modifiers_2 = modifiers.clone();
    event_controller.connect_key_pressed(move |_, key, _, _| {
        handle_key(
            key,
            &plugin_keys,
            open_modifier,
            modifiers_2.clone(),
            event_sender_3.clone(),
        )
    });
    event_controller.connect_key_released(move |_, key, _, _| {
        handle_release(key, modifiers.clone());
    });
    event_controller.set_propagation_phase(PropagationPhase::Capture);
    entry.add_controller(event_controller);
    let entry_2 = entry.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        entry_2.grab_focus_without_selecting(); // ensure that the entry is always focused
        ControlFlow::Continue
    });
    main_vbox.append(&entry);

    let results = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["launcher-results"])
        .spacing(3)
        .build();
    main_vbox.append(&results);

    let plugin_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["launcher-plugins"])
        .spacing(5)
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
    window.set_exclusive_zone(-1);
    window.present();
    window.set_visible(false);

    debug!("Created launcher window ({})", window.id());

    launcher_entry_text_change("".to_string(), event_sender);

    Ok(LauncherData {
        config: LauncherConfig {
            default_terminal: launcher.default_terminal.clone(),
            max_items: launcher.max_items,
            show_when_empty: launcher.show_when_empty,
            animate_launch_ms: launcher.animate_launch_ms,
            width: launcher.width,
            data_dir: PathBuf::from(data_dir).into_boxed_path(),
            plugins: launcher.plugins.clone(),
        },
        window,
        entry,
        results,
        plugin_box,
        sorted_matches: vec![],
        static_matches: HashMap::new(),
    })
}

fn launcher_entry_text_change(text: String, event_sender: Sender<TransferType>) {
    event_sender
        .send_blocking(TransferType::Type(text))
        .warn("unable to send");
}

fn handle_release(key: Key, mods: Arc<Mutex<u16>>) {
    let mut mods = mods.lock().unwrap();
    match key {
        Key::Shift_L | Key::Shift_R => *mods &= !1,
        Key::Control_L | Key::Control_R => *mods &= !2,
        Key::Alt_L | Key::Alt_R => *mods &= !4,
        Key::Super_L | Key::Super_R => *mods &= !8,
        _ => (),
    };
    // trace!("key: {}{:?}, mods: {}", key, key, mods);
}

fn handle_key(
    key: Key,
    plugin_keys: &[Key],
    open_modifier: Mod,
    mods: Arc<Mutex<u16>>,
    event_sender: Sender<TransferType>,
) -> Propagation {
    let mut mods = mods.lock().unwrap();
    match key {
        Key::Shift_L | Key::Shift_R => *mods |= 1,
        Key::Control_L | Key::Control_R => *mods |= 2,
        Key::Alt_L | Key::Alt_R => *mods |= 4,
        Key::Super_L | Key::Super_R => *mods |= 8,
        _ => (),
    };
    // plugins always use ctrl modifier
    if *mods == 2 && plugin_keys.contains(&key) {
        let ch = key
            .name()
            .unwrap_or_default()
            .to_string()
            .pop()
            .unwrap_or('a');
        event_sender
            .send_blocking(TransferType::CloseOverview(
                CloseOverviewConfig::LauncherPress(ch),
            ))
            .warn("unable to send");
        return Propagation::Stop;
    }

    if ((key == Key::Alt_L || key == Key::Alt_R) && open_modifier == Mod::Alt)
        || ((key == Key::Control_L || key == Key::Control_R) && open_modifier == Mod::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && open_modifier == Mod::Super)
    {
        event_sender
            .send_blocking(TransferType::Exit)
            .warn("unable to send");
        return Propagation::Stop;
    }

    match key {
        Key::Escape => {
            event_sender
                .send_blocking(TransferType::Exit)
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Tab => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Right,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::ISO_Left_Tab => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Left,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::grave => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Left,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Return => {
            event_sender
                .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::None))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Up => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Up,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Down => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Down,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Left => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Left,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::Right => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Right,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_1 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('1'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_2 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('2'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_3 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('3'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_4 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('4'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_5 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('5'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_6 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('6'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_7 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('7'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_8 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('8'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }
        Key::_9 => {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress('9'),
                ))
                .warn("unable to send");
            Propagation::Stop
        }

        _ => Propagation::Proceed,
    }
}
