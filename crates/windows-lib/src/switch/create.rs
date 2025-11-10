use crate::global::{WindowsSwitchConfig, WindowsSwitchData};
use anyhow::Context;
use async_channel::Sender;
use config_lib::{FilterBy, Modifier, Switch, Windows};
use core_lib::transfer::{CloseSwitchConfig, Direction, SwitchSwitchConfig, TransferType};
use core_lib::{HyprlandData, SWITCH_NAMESPACE, WarnWithDetails};
use exec_lib::get_initial_active;
use adw::gtk::gdk::{Key, ModifierType};
use adw::gtk::glib::Propagation;
use adw::gtk::prelude::*;
use adw::gtk::{
    Application, ApplicationWindow, EventControllerKey, FlowBox, Orientation, Overlay,
    SelectionMode,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
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

    let key_controller = EventControllerKey::new();
    let event_sender_2 = event_sender.clone();
    let switch_key = switch.key.clone();
    let modifier = switch.modifier;
    key_controller.connect_key_pressed(move |_, key, _, mods| handle_key(key, mods, &switch_key, modifier, &event_sender_2));
    let event_sender_3 = event_sender;
    let switch_key_2 = switch.key.clone();
    key_controller.connect_key_released(move |_, key, _, _| {
        handle_release(key, &switch_key_2, modifier, &event_sender_3);
    });
    window.add_controller(key_controller);

    window.init_layer_shell();
    window.set_namespace(Some(SWITCH_NAMESPACE));
    window.set_layer(Layer::Top);
    // we only have one window, so we can do this
    // we also don't use adw::gtk::Popover which doesnt work with exclusive mode
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.present();
    window.set_visible(false);

    debug!("Created switch window ({})", window.id());

    Ok(WindowsSwitchData {
        config: WindowsSwitchConfig {
            items_per_row: windows.items_per_row,
            scale: windows.scale,
            filter_current_workspace: switch.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_current_monitor: switch.filter_by.contains(&FilterBy::CurrentMonitor),
            filter_same_class: switch.filter_by.contains(&FilterBy::SameClass),
            switch_workspaces: switch.switch_workspaces,
            key: switch.key.clone(),
            modifier: switch.modifier.to_string().to_lowercase().into(),
        },
        window,
        main_flow: clients_flow,
        workspaces: HashMap::default(),
        clients: HashMap::default(),
        active: get_initial_active().context("unable to get initial active data")?,
        hypr_data: HyprlandData::default(),
    })
}

fn handle_release(key: Key, switch_key: &Box<str>, switch_mod: Modifier, event_sender: &Sender<TransferType>) {
    if ((key == Key::Alt_L || key == Key::Alt_R) && switch_mod == Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R) && switch_mod == Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && switch_mod == Modifier::Super)
    {
        event_sender
            .send_blocking(TransferType::CloseSwitch(CloseSwitchConfig { 
                modifier: switch_mod.to_string().to_lowercase().into(),
                key: switch_key.clone(),
            }))
            .warn_details("unable to send");
    }
}

fn handle_key(key: Key, mods: ModifierType, switch_key: &Box<str>, switch_mod: Modifier, event_sender: &Sender<TransferType>) -> Propagation {
    let s_key = if &**switch_key == "tab" { "Tab" } else { switch_key };
    let k = Key::from_name(s_key);
    let mm = match switch_mod {
        Modifier::Alt => ModifierType::ALT_MASK,
        Modifier::Ctrl => ModifierType::CONTROL_MASK,
        Modifier::Super => ModifierType::SUPER_MASK,
    };
    // TODO: handle arrow keys and hjkl
    // a SwitchSwitch maybe doesn't need to take care of mod/key checks?!
    /*
    fn handle_key(key: Key, event_sender: &Sender<TransferType>) -> Propagation {
    match key {
        Key::Tab | Key::l | Key::Right => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    direction: Direction::Right,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        Key::ISO_Left_Tab | Key::grave | Key::dead_grave | Key::h | Key::Left => {
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
        _ => Propagation::Proceed,
     */
    if let Some(k) = k {
        match key {
            x if x.eq(&k) && mods.contains(mm) => {
                event_sender
                    .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                        modifier: switch_mod.to_string().to_lowercase().into(),
                        key: switch_key.clone(),
                        direction: if mods.contains(ModifierType::SHIFT_MASK) {
                            Direction::Left
                        } else {
                            Direction::Right
                        },
                    }))
                    .warn_details("unable to send");
                Propagation::Stop
            }
            _ => Propagation::Proceed,
        }
    } else {
        warn!("Unable to find key name: {}", switch_key);
        Propagation::Proceed
    }
}
