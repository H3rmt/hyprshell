use crate::data::{collect_data, SortConfig};
use crate::icon::set_icon;
use crate::next::find_next;
use crate::WindowsGlobal;
use anyhow::Context;
use core_lib::transfer::{CloseConfig, OpenSwitch, TransferType, WindowsOverride};
use core_lib::{send_to_socket, ClientData, ClientId, FindByFirst, Warn};
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{pango, Button, Frame, Image, Label, Overflow, Overlay};
use tracing::{debug, span, trace, Level};

fn scale(value: i16, scale: f64) -> i32 {
    (value as f64 / (15f64 - scale)) as i32
}

pub fn open_switch(global: &WindowsGlobal, config: OpenSwitch) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "open_switch").entered();
    let (clients_data, active_prev) = collect_data(&SortConfig {
        filter_current_monitor: config.filter_current_monitor,
        filter_current_workspace: config.filter_current_workspace,
        filter_same_class: config.filter_same_class,
        sort_recent: true,
    })
    .context("Failed to collect data")?;
    let active = find_next(
        &config.direction,
        false,
        &clients_data,
        active_prev,
        global.workspaces_per_row as usize,
    );

    let mut data = global.data.borrow_mut();
    for (window, monitor_data) in &mut data.monitor_list.iter_mut() {
        if monitor_data.id != active_prev.monitor {
            continue;
        }
        let monitor = if let Some(monitor) = clients_data.monitors.find_by_first(&monitor_data.id) {
            monitor
        } else {
            continue;
        };
        trace!("Showing window {:?}", window.id());
        window.set_visible(true);

        let clients: &Vec<(ClientId, ClientData)> = &clients_data.clients;
        for (address, client) in clients {
            if config.hide_filtered && !client.enabled {
                continue;
            }
            let client_button = {
                let title = if !client.title.trim().is_empty() {
                    &client.title
                } else {
                    &client.class
                };
                let client_label = Label::builder()
                    .label(title)
                    .overflow(Overflow::Visible)
                    .margin_start(6)
                    .ellipsize(pango::EllipsizeMode::End)
                    .build();
                let client_frame = Frame::builder()
                    .label_xalign(0.5)
                    .label_widget(&client_label)
                    .build();

                // hide picture if client so small
                let client_h_w = scale(monitor.height as i16, global.scale)
                    .min(scale(monitor.width as i16, global.scale));
                if client_h_w > 70 {
                    let image = Image::builder()
                        .css_classes(["client-image"])
                        .pixel_size((client_h_w.clamp(50, 600) as f64 / 1.5) as i32 - 20)
                        .build();
                    if !client.enabled {
                        image.add_css_class("monochrome");
                    }
                    set_icon(&client.class, client.pid, &image);
                    client_frame.set_child(Some(&image));
                }

                let client_overlay = Overlay::builder()
                    .overflow(Overflow::Hidden)
                    .child(&client_frame)
                    .build();
                let button = Button::builder()
                    .child(&client_overlay)
                    .css_classes(["client"])
                    .width_request(scale(monitor.width as i16, global.scale))
                    .height_request(scale(monitor.height as i16, global.scale))
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                // add border around initial active client
                if active.client == Some(*address) {
                    button.add_css_class("active");
                }

                click_client(&button, *address);
                button
            };
            monitor_data.workspaces_flow.insert(&client_button, -1);
            monitor_data.client_refs.insert(*address, client_button);
        }
        //        trace!("Refresh window {:?}", window.id());
    }

    data.active = active;
    data.hypr_data = clients_data;
    drop(data);
    Ok(())
}

fn click_client(button: &Button, client_id: ClientId) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of client box");
        send_to_socket(&TransferType::Close(CloseConfig::Windows(
            WindowsOverride::ClientId(client_id),
        )))
        .warn("unable send return to socket");
    });
}
