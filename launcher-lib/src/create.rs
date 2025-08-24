use crate::global::{LauncherConfig, LauncherData};
use crate::plugins::get_static_options_chars;
use async_channel::Sender;
use config_lib::{Launcher, Modifier};
use core_lib::transfer::{CloseOverviewConfig, Direction, SwitchOverviewConfig, TransferType};
use core_lib::{LAUNCHER_NAMESPACE, WarnWithDetails};
use gtk::gdk::{Key, ModifierType};
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
use tracing::{debug, debug_span, trace};

pub fn create_windows_overview_launcher_window(
    app: &Application,
    launcher: &Launcher,
    open_modifier: Modifier,
    open_key: &Box<str>,
    data_dir: &Path,
    event_sender: &Sender<TransferType>,
) -> anyhow::Result<LauncherData> {
    let _span = debug_span!("create_windows_overview_launcher_window").entered();

    let main_vbox = ListBox::builder()
        .css_classes(["launcher"])
        .width_request(i32::try_from(launcher.width)?)
        .selection_mode(SelectionMode::None)
        .build();

    let entry = Entry::builder().css_classes(["launcher-input"]).build();
    let event_sender_2 = event_sender.clone();
    entry.connect_changed(move |e| {
        launcher_entry_text_change(e.text().to_string(), &event_sender_2);
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
        .spacing(4)
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
    window.set_margin(Edge::Top, 17);
    window.set_exclusive_zone(-1);
    window.present();
    window.set_visible(false);

    let event_controller = EventControllerKey::new();
    let plugin_keys = get_static_options_chars(&launcher.plugins);
    let event_sender_4 = event_sender.clone();
    let entry_2 = entry.clone();
    let results_2 = results.clone();
    let launch_modifier = launcher.launch_modifier;
    event_controller.connect_key_pressed(move |_, key, _, modt| {
        handle_key(
            &entry_2,
            key,
            open_modifier,
            modt,
            &plugin_keys,
            launch_modifier,
            &results_2,
            &event_sender_4,
        )
    });
    event_controller.set_propagation_phase(PropagationPhase::Capture);
    entry.add_controller(event_controller);
    let entry_2 = entry.clone();
    let window_2 = window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if window_2.is_visible() {
            entry_2.grab_focus_without_selecting(); // ensure that the entry is always focused
        }
        ControlFlow::Continue
    });

    debug!("Created launcher window ({})", window.id());

    launcher_entry_text_change(String::new(), event_sender);

    Ok(LauncherData {
        config: LauncherConfig {
            default_terminal: launcher.default_terminal.clone(),
            max_items: launcher.max_items,
            launch_modifier: launcher.launch_modifier,
            show_when_empty: launcher.show_when_empty,
            width: launcher.width,
            data_dir: PathBuf::from(data_dir).into_boxed_path(),
            plugins: launcher.plugins.clone(),
        },
        window,
        entry,
        results_box: results,
        results_items: HashMap::new(),
        plugins_box: plugin_box,
        plugins_items: HashMap::new(),
        sorted_matches: vec![],
        static_matches: HashMap::new(),
    })
}

fn launcher_entry_text_change(text: String, event_sender: &Sender<TransferType>) {
    event_sender
        .send_blocking(TransferType::Type(text))
        .warn_details("unable to send");
}

// TODO this doesnt work if key is not a modifier (tab instead of super_l)
// instead handle close with esc and close with mod again in plugin
fn handle_release(
    key: Key,
    modifier: Modifier,
    open_key: &Box<str>,
    mt: ModifierType,
    event_sender: &Sender<TransferType>,
) {
    if ((key == Key::Alt_L || key == Key::Alt_R) && modifier == Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R) && modifier == Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && modifier == Modifier::Super)
    {
        trace!("Modifier key released: {:?}", key);
        event_sender
            .send_blocking(TransferType::Exit)
            .warn_details("unable to send");
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn handle_key(
    entry: &Entry,
    key: Key,
    modifier: Modifier,
    modt: ModifierType,
    plugin_keys: &[Key],
    launch_modifier: Modifier,
    results: &gtk::Box,
    event_sender: &Sender<TransferType>,
) -> Propagation {
    let launch_mod = match launch_modifier {
        Modifier::Ctrl => modt == ModifierType::CONTROL_MASK,
        Modifier::Alt => modt == ModifierType::ALT_MASK,
        Modifier::Super => modt == ModifierType::SUPER_MASK,
    };
    // tracing::trace!(
    //     "key: {}{:?}, mods: {:?}, launch_mod: {}, launch_modifier: {}",
    //     key,
    //     key,
    //     modt,
    //     launch_mod,
    //     launch_modifier
    // );
    if launch_mod && plugin_keys.contains(&key) {
        if let Some(ch) = key.name().unwrap_or_default().to_string().pop() {
            event_sender
                .send_blocking(TransferType::CloseOverview(
                    CloseOverviewConfig::LauncherPress(ch),
                ))
                .warn_details("unable to send");
        }
        return Propagation::Stop;
    }

    match (launch_mod, key) {
        (_, Key::Escape) => {
            event_sender
                .send_blocking(TransferType::Exit)
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Tab) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Right,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::ISO_Left_Tab | Key::grave) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Left,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Up) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Up,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Down) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Down,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Left) => {
            if !entry.text().is_empty() {
                // allow to use in text in launcher
                return Propagation::Proceed;
            }
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Left,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Right) => {
            if !entry.text().is_empty() {
                // allow to use in text in launcher
                return Propagation::Proceed;
            }
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Right,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        (_, Key::Return) => {
            if results.first_child().is_some() {
                event_sender
                    .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::None))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_1) => {
            if results.observe_children().into_iter().len() > 1 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('1'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_2) => {
            if results.observe_children().into_iter().len() > 2 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('2'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_3) => {
            if results.observe_children().into_iter().len() > 3 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('3'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_4) => {
            if results.observe_children().into_iter().len() > 4 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('4'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_5) => {
            if results.observe_children().into_iter().len() > 5 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('5'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_6) => {
            if results.observe_children().into_iter().len() > 6 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('6'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_7) => {
            if results.observe_children().into_iter().len() > 7 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('7'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_8) => {
            if results.observe_children().into_iter().len() > 8 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('8'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        (true, Key::_9) => {
            if results.observe_children().into_iter().len() > 9 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('9'),
                    ))
                    .warn_details("unable to send");
            }
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    }
}
