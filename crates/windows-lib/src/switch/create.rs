use crate::global::{WindowsSwitchConfig, WindowsSwitchData};
use anyhow::Context;
use async_channel::Sender;
use config_lib::{FilterBy, KeyCombo, KeyMod, Switch, Windows};
use core_lib::transfer::{Direction, HoldMod, SwitchSwitchConfig, TransferType};
use core_lib::{HyprlandData, SWITCH_NAMESPACE, WarnWithDetails};
use exec_lib::get_initial_active;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use relm4::adw::gtk::gdk::Key;
use relm4::adw::gtk::glib::Propagation;
use relm4::adw::gtk::prelude::*;
use relm4::adw::gtk::{
    Application, ApplicationWindow, EventControllerKey, FlowBox, Orientation, Overlay,
    SelectionMode,
};
use std::collections::HashMap;
use tracing::{debug, debug_span, warn};

pub fn create_windows_switch_window(
    app: &Application,
    switch: &Switch,
    windows: &Windows,
    event_sender: Sender<TransferType>,
) -> anyhow::Result<WindowsSwitchData> {
    let _span = debug_span!("create_windows_switch_window").entered();

    let clients_flow = FlowBox::builder()
        .selection_mode(SelectionMode::None)
        .orientation(Orientation::Horizontal)
        .max_children_per_line(u32::from(windows.items_per_row))
        .min_children_per_line(u32::from(windows.items_per_row))
        .build();

    let clients_flow_overlay = Overlay::builder()
        .child(&clients_flow)
        .css_classes(["monitor", "no-hover"])
        .build();

    let window = ApplicationWindow::builder()
        .css_classes(["window"])
        .application(app)
        .child(&clients_flow_overlay)
        .default_height(10)
        .default_width(10)
        .build();

    let forward_keys = collect_keys(&switch.binds.forward);
    let reverse_keys = collect_keys(&switch.binds.reverse);
    if forward_keys.is_empty() {
        warn!("Switch profile has no valid forward keys, navigation will rely on arrow/vim keys");
    }
    if reverse_keys.is_empty() {
        warn!("Switch profile has no valid reverse keys, navigation will rely on arrow/vim keys");
    }
    let kill_key = Key::from_name(switch.kill_key.to_string()).context("invalid kill key")?;
    let active_hold_mods = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let key_controller = EventControllerKey::new();
    let event_sender_2 = event_sender.clone();
    let forward_keys_2 = forward_keys.clone();
    let reverse_keys_2 = reverse_keys.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        handle_key(key, &forward_keys_2, &reverse_keys_2, kill_key, &event_sender_2)
    });
    let event_sender_3 = event_sender;
    let hold_mods = active_hold_mods.clone();
    key_controller.connect_key_released(move |_, key, _, _| {
        handle_release(key, &hold_mods, &event_sender_3);
    });
    window.add_controller(key_controller);

    window.init_layer_shell();
    window.set_namespace(Some(SWITCH_NAMESPACE));
    window.set_layer(Layer::Top);
    // we only have one window, so we can do this
    // we also don't use relm4::adw::gtk::Popover which doesnt work with exclusive mode
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    debug!("Created switch window ({})", window.id());

    Ok(WindowsSwitchData {
        config: WindowsSwitchConfig {
            items_per_row: windows.items_per_row,
            scale: windows.scale,
            filter_current_workspace: switch.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_current_monitor: switch.filter_by.contains(&FilterBy::CurrentMonitor),
            filter_same_class: switch.filter_by.contains(&FilterBy::SameClass),
            switch_workspaces: switch.switch_workspaces,
            exclude_workspaces: if switch.exclude_workspaces.is_empty() {
                None
            } else {
                Some(switch.exclude_workspaces.clone())
            },
        },
        active_hold_mods,
        window,
        main_flow: clients_flow,
        workspaces: HashMap::default(),
        clients: HashMap::default(),
        active: get_initial_active().context("unable to get initial active data")?,
        hypr_data: HyprlandData::default(),
    })
}

fn handle_release(
    key: Key,
    hold_mods: &std::rc::Rc<std::cell::RefCell<Vec<HoldMod>>>,
    event_sender: &Sender<TransferType>,
) {
    if !matches_hold_mod(key, &hold_mods.borrow()) {
        return;
    }
    event_sender
        .send_blocking(TransferType::CloseSwitch)
        .warn_details("unable to send");
}

fn handle_key(
    key: Key,
    forward_keys: &[Key],
    reverse_keys: &[Key],
    kill_key: Key,
    event_sender: &Sender<TransferType>,
) -> Propagation {
    match key {
        Key::Escape => {
            event_sender
                .send_blocking(TransferType::CloseAll)
                .warn_details("unable to send");
            Propagation::Stop
        }
        k if forward_keys.contains(&k) || k == Key::l || k == Key::Right => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    direction: Direction::Right,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        k if reverse_keys.contains(&k) || k == Key::h || k == Key::Left => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    direction: Direction::Left,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        Key::j | Key::Down => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    direction: Direction::Down,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        Key::k | Key::Up => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    direction: Direction::Up,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        k if k == kill_key || k == Key::Delete => {
            event_sender
                .send_blocking(TransferType::CloseClientSwitch)
                .warn_details("unable to send");
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    }
}

fn collect_keys(keys: &[KeyCombo]) -> Vec<Key> {
    let mut collected = Vec::new();
    for combo in keys {
        if combo
            .key
            .as_ref()
            .eq_ignore_ascii_case("tab")
            && combo.mods.iter().any(|m| *m == KeyMod::Shift)
        {
            push_unique_key(&mut collected, Key::ISO_Left_Tab);
            continue;
        }
        if combo.key.as_ref().eq_ignore_ascii_case("grave") {
            push_unique_key(&mut collected, Key::grave);
            push_unique_key(&mut collected, Key::dead_grave);
            continue;
        }
        if let Some(key) = Key::from_name(combo.key.as_ref()) {
            push_unique_key(&mut collected, key);
        } else {
            warn!("Invalid switch key: {}", combo.key);
        }
    }
    collected
}

fn push_unique_key(keys: &mut Vec<Key>, key: Key) {
    if !keys.contains(&key) {
        keys.push(key);
    }
}

fn matches_hold_mod(key: Key, hold_mods: &[HoldMod]) -> bool {
    for hold in hold_mods {
        match hold {
            HoldMod::Alt if key == Key::Alt_L || key == Key::Alt_R => return true,
            HoldMod::Ctrl if key == Key::Control_L || key == Key::Control_R => return true,
            HoldMod::Super if key == Key::Super_L || key == Key::Super_R => return true,
            _ => {}
        }
    }
    false
}
