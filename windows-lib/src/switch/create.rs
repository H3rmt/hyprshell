use crate::global::{WindowsSwitchConfig, WindowsSwitchData};
use async_channel::Sender;
use config_lib::{FilterBy, Modifier, Switch, Windows};
use core_lib::transfer::{SwitchSwitchConfig, TransferType};
use core_lib::{HyprlandData, SWITCH_NAMESPACE, WarnWithDetails};
use exec_lib::get_initial_active;
use gtk::gdk::Key;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, EventControllerKey, FlowBox, Orientation, Overlay,
    SelectionMode,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::collections::HashMap;
use tracing::{debug, debug_span};

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
    key_controller.connect_key_pressed(move |_, key, _, _| handle_key(key, &event_sender_2));
    let event_sender_3 = event_sender;
    let r#mod = switch.modifier;
    key_controller.connect_key_released(move |_, key, _, _| {
        handle_release(key, r#mod, &event_sender_3);
    });
    window.add_controller(key_controller);

    window.init_layer_shell();
    window.set_namespace(Some(SWITCH_NAMESPACE));
    window.set_layer(Layer::Top);
    window.set_can_focus(false);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.present();
    window.set_visible(false);

    debug!("Created switch window ({})", window.id(),);

    Ok(WindowsSwitchData {
        config: WindowsSwitchConfig {
            items_per_row: windows.items_per_row,
            scale: windows.scale,
            filter_current_workspace: switch.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_current_monitor: switch.filter_by.contains(&FilterBy::CurrentMonitor),
            filter_same_class: switch.filter_by.contains(&FilterBy::SameClass),
            switch_workspaces: switch.switch_workspaces,
        },
        window,
        main_flow: clients_flow,
        workspaces: HashMap::default(),
        clients: HashMap::default(),
        active: get_initial_active()?,
        hypr_data: HyprlandData::default(),
    })
}

fn handle_release(key: Key, modifier: Modifier, event_sender: &Sender<TransferType>) {
    if ((key == Key::Alt_L || key == Key::Alt_R) && modifier == Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R) && modifier == Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && modifier == Modifier::Super)
    {
        event_sender
            .send_blocking(TransferType::CloseSwitch)
            .warn_details("unable to send");
    }
}

fn handle_key(key: Key, event_sender: &Sender<TransferType>) -> Propagation {
    match key {
        Key::Tab => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    reverse: false,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        Key::ISO_Left_Tab | Key::grave => {
            event_sender
                .send_blocking(TransferType::SwitchSwitch(SwitchSwitchConfig {
                    reverse: true,
                }))
                .warn_details("unable to send");
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    }
}
