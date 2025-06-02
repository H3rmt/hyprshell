use crate::WindowsGlobal;
use crate::global::OverviewGlobalMonitorData;
use anyhow::Context;
use core_lib::OVERVIEW_NAMESPACE;
use exec_lib::get_monitors;
use gtk::gdk::{Display, Monitor};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, FlowBox, Orientation, Overlay, SelectionMode};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use tracing::{Level, debug, span};

pub fn create_windows_window(app: &Application, global: &WindowsGlobal) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "create_windows_window").entered();
    let monitors = get_monitors();
    if let Ok(display) = Display::default().context("Could not connect to a display") {
        let gtk_monitors = display
            .monitors()
            .iter()
            .filter_map(|m| m.ok())
            .collect::<Vec<Monitor>>();

        for monitor in gtk_monitors {
            let monitor_name = monitor.connector().unwrap_or_default();
            let monitor_id = monitors
                .iter()
                .find(|m| m.name == monitor_name)
                .map(|m| m.id)
                .unwrap_or_default();

            let workspaces_flow = FlowBox::builder()
                .selection_mode(SelectionMode::None)
                .orientation(Orientation::Horizontal)
                .max_children_per_line(global.workspaces_per_row as u32)
                .min_children_per_line(global.workspaces_per_row as u32)
                .build();

            let workspaces_flow_overlay = Overlay::builder()
                .child(&workspaces_flow)
                .css_classes(["monitor"])
                .build();

            let window = ApplicationWindow::builder()
                .css_classes(["window"])
                .application(app)
                .child(&workspaces_flow_overlay)
                .default_height(10)
                .default_width(10)
                .build();

            window.init_layer_shell();
            window.set_namespace(Some(OVERVIEW_NAMESPACE));
            window.set_layer(Layer::Overlay);
            window.set_keyboard_mode(KeyboardMode::Exclusive);
            window.set_can_focus(false);
            window.set_monitor(Some(&monitor));
            window.present();
            window.set_visible(false);

            debug!(
                "Created window window ({}) for monitor {:?}",
                window.id(),
                monitor_name
            );
            global.data.borrow_mut().monitor_list.insert(
                window,
                OverviewGlobalMonitorData::new(monitor_id, workspaces_flow),
            );
        }
    }

    Ok(())
}
