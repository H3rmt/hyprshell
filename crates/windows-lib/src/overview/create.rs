use crate::global::{WindowsOverviewConfig, WindowsOverviewData, WindowsOverviewMonitorData};
use anyhow::Context;
use config_lib::{FilterBy, Overview, Windows};
use core_lib::{HyprlandData, OVERVIEW_NAMESPACE};
use exec_lib::{collect, get_initial_active};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::adw::gtk::gdk::{Display, Monitor};
use relm4::adw::gtk::prelude::*;
use relm4::adw::gtk::{
    Application, ApplicationWindow, FlowBox, Orientation, Overlay, SelectionMode,
};
use std::collections::HashMap;
use tracing::{debug, debug_span};

pub fn create_windows_overview_window(
    app: &Application,
    overview: &Overview,
    windows: &Windows,
) -> anyhow::Result<WindowsOverviewData> {
    let _span = debug_span!("create_windows_overview_window").entered();
    let mut window_list = HashMap::new();

    let monitors = collect::get_monitors();
    if let Ok(display) = Display::default().context("Could not connect to a display") {
        let gtk_monitors = display
            .monitors()
            .iter()
            .filter_map(Result::ok)
            .collect::<Vec<Monitor>>();

        for gtk_monitor in gtk_monitors {
            let monitor_name = gtk_monitor.connector().unwrap_or_default();
            if let Some(monitor) = monitors.iter().find(|m| m.connector == monitor_name) {
                let workspaces_flow = FlowBox::builder()
                    .selection_mode(SelectionMode::None)
                    .orientation(Orientation::Horizontal)
                    .max_children_per_line(u32::from(windows.general.items_per_row))
                    .min_children_per_line(u32::from(windows.general.items_per_row))
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
                window.set_layer(Layer::Top);
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, 430i32);
                window.set_keyboard_mode(KeyboardMode::Exclusive);
                window.set_monitor(Some(&gtk_monitor));

                debug!(
                    "Created overview window ({}) for monitor {monitor_name:?}",
                    window.id()
                );
                window_list.insert(
                    window,
                    WindowsOverviewMonitorData::new(monitor.id, workspaces_flow),
                );
            }
        }
    }

    let active = get_initial_active().context("unable to get initial active data")?;
    Ok(WindowsOverviewData {
        config: WindowsOverviewConfig {
            items_per_row: windows.general.items_per_row,
            scale: windows.general.scale,
            filter_current_workspace: overview.filter_by.contains(&FilterBy::CurrentWorkspace),
            filter_current_monitor: overview.filter_by.contains(&FilterBy::CurrentMonitor),
            filter_same_class: overview.filter_by.contains(&FilterBy::SameClass),
            exclude_workspaces: if overview.exclude_workspaces.is_empty() {
                None
            } else {
                Some(overview.exclude_workspaces.clone())
            },
        },
        window_list,
        active,
        initial_active: active,
        hypr_data: HyprlandData::default(),
    })
}
