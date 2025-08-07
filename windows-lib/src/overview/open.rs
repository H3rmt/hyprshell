use crate::data::{SortConfig, collect_data};
use crate::global::WindowsOverviewData;
use crate::icon::set_icon;
use anyhow::Context;
use async_channel::Sender;
use core_lib::transfer::{CloseOverviewConfig, TransferType, WindowsOverride};
use core_lib::{ClientData, ClientId, WarnWithDetails, WorkspaceId};
use exec_lib::set_remain_focused;
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use std::borrow::Cow;
use tracing::{Level, debug, debug_span, span, trace};

fn scale(value: i16, scale: f64) -> i32 {
    (value as f64 / (15f64 - scale)) as i32
}

pub fn overview_already_open(data: &WindowsOverviewData) -> bool {
    data.window_list.iter().any(|w| w.0.get_visible())
}

pub fn open_overview(
    data: &mut WindowsOverviewData,
    event_sender: Sender<TransferType>,
) -> anyhow::Result<()> {
    let _span = debug_span!("open_overview").entered();
    set_remain_focused().warn_details("Failed to set no follow mouse");

    let (clients_data, active) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: false,
    })
    .context("Failed to collect data")?;
    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    for (window, monitor_data) in data.window_list.iter_mut() {
        trace!("Showing window {:?}", window.id());
        window.set_visible(true);

        let workspaces = {
            let mut workspaces = clients_data
                .workspaces
                .iter()
                .filter(|(_, v)| v.monitor == monitor_data.id)
                .collect::<Vec<_>>();
            workspaces.sort_by(|(a, _), (b, _)| a.cmp(b));
            workspaces
        };

        for (wid, workspace) in workspaces {
            trace!(
                "Creating workspace {:?} with ({}x{})",
                wid,
                scale(workspace.width as i16, data.config.scale),
                scale(workspace.height as i16, data.config.scale)
            );
            let workspace_fixed = Fixed::builder()
                .width_request(scale(workspace.width as i16, data.config.scale))
                .height_request(scale(workspace.height as i16, data.config.scale))
                .build();
            let id_string = wid.to_string();
            let title = if !workspace.name.trim().is_empty() {
                remove_html.replace_all(&workspace.name, "")
            } else {
                Cow::from(&id_string)
            };

            let workspace_frame = Frame::builder()
                .label(title)
                .label_xalign(0.5)
                .child(&workspace_fixed)
                .build();

            let workspace_button = {
                let workspace_overlay = Overlay::builder().child(&workspace_frame).build();
                let button = gtk::Box::builder().css_classes(["workspace"]).build();
                button.append(&workspace_overlay);
                if active.client.is_none() && active.workspace == *wid {
                    button.add_css_class("active");
                }
                monitor_data.workspaces_flow.insert(&button, -1);
                button
            };
            monitor_data.workspaces.insert(*wid, workspace_button);

            let clients: Vec<&(ClientId, ClientData)> = {
                let mut clients = clients_data
                    .clients
                    .iter()
                    .filter(|(_, client)| {
                        client.workspace == *wid && (!data.config.hide_filtered || client.enabled)
                    })
                    .collect::<Vec<_>>();
                clients.sort_by(|(_, a), (_, b)| {
                    // prefer smaller windows
                    if a.floating && b.floating {
                        (b.width * b.height).cmp(&(a.width * a.height))
                    } else {
                        a.floating.cmp(&b.floating)
                    }
                });
                clients
            };
            for (address, client) in clients {
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
                    let client_h_w = scale(client.height, data.config.scale)
                        .min(scale(client.width, data.config.scale));
                    if client_h_w > 70 {
                        let image = Image::builder()
                            .css_classes(["client-image"])
                            .pixel_size((client_h_w.clamp(50, 600) as f64 / 1.6) as i32 - 20)
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
                        .width_request(scale(client.width, data.config.scale))
                        .height_request(scale(client.height, data.config.scale))
                        .build();
                    button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                    // add initial border around initial active client
                    if active.client == Some(*address) {
                        button.add_css_class("active");
                    }

                    click_client(&button, *address, event_sender.clone());
                    button
                };
                trace!(
                    "Creating Client {:?} with ({}x{}) at ({}x{})",
                    address,
                    scale(client.width, data.config.scale),
                    scale(client.height, data.config.scale),
                    scale(client.x - workspace.x as i16, data.config.scale) as f64,
                    scale(client.y - workspace.y as i16, data.config.scale) as f64
                );
                workspace_fixed.put(
                    &client_button,
                    scale(client.x - workspace.x as i16, data.config.scale) as f64,
                    scale(client.y - workspace.y as i16, data.config.scale) as f64,
                );
                monitor_data.clients.insert(*address, client_button);
            }
        }
    }

    data.active = active;
    data.initial_active = active;
    data.hypr_data = clients_data;
    Ok(())
}

fn click_client(button: &Button, client_id: ClientId, event_sender: Sender<TransferType>) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of client button");
        event_sender
            .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::Windows(
                WindowsOverride::ClientId(client_id),
            )))
            .warn_details("unable to send");
    });
}
