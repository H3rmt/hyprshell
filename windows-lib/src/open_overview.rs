use crate::WindowsGlobal;
use crate::data::{SortConfig, collect_data};
use crate::icon::set_icon;
use anyhow::Context;
use core_lib::transfer::{CloseConfig, OpenOverview, TransferType, WindowsOverride};
use core_lib::{ClientData, ClientId, FindByFirst, Warn, WorkspaceId, send_to_socket};
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use std::borrow::Cow;
use tracing::{Level, debug, span, trace};

fn scale(value: i16, scale: f64) -> i32 {
    (value as f64 / (15f64 - scale)) as i32
}

pub fn open_overview(global: &WindowsGlobal, config: OpenOverview) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "open_overview").entered();
    let (clients_data, active) = collect_data(&SortConfig {
        filter_current_monitor: config.filter_current_monitor,
        filter_current_workspace: config.filter_current_workspace,
        filter_same_class: config.filter_same_class,
        sort_recent: false,
    })
    .context("Failed to collect data")?;

    let regex = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    let mut data = global.data.borrow_mut();
    for (window, monitor_data) in data.monitor_list.iter_mut() {
        if config.hide_filtered
            && clients_data
                .monitors
                .find_by_first(&monitor_data.id)
                .map(|m| !m.enabled)
                .unwrap_or(false)
        {
            continue;
        }
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
                scale(workspace.width as i16, global.scale),
                scale(workspace.height as i16, global.scale)
            );
            let workspace_fixed = Fixed::builder()
                .width_request(scale(workspace.width as i16, global.scale))
                .height_request(scale(workspace.height as i16, global.scale))
                .build();

            let id_string = wid.to_string();
            let title = if !workspace.name.trim().is_empty() {
                if global.strip_html_from_workspace_title {
                    regex.replace_all(&workspace.name, "")
                } else {
                    Cow::from(&workspace.name)
                }
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
                let button = Button::builder()
                    .child(&workspace_overlay)
                    .css_classes(["workspace"])
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());
                click_workspace(&button, *wid);
                monitor_data.workspaces_flow.insert(&button, -1);
                button
            };
            monitor_data.workspace_refs.insert(*wid, workspace_button);

            let clients: Vec<&(ClientId, ClientData)> = {
                let mut clients = clients_data
                    .clients
                    .iter()
                    .filter(|(_, client)| client.workspace == *wid)
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
                    let client_h_w =
                        scale(client.height, global.scale).min(scale(client.width, global.scale));
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
                        .width_request(scale(client.width, global.scale))
                        .height_request(scale(client.height, global.scale))
                        .build();
                    button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                    // add initial border around initial active client
                    if active.client == Some(*address) {
                        button.add_css_class("active");
                    }

                    click_client(&button, *address);
                    button
                };
                trace!(
                    "Creating Client {:?} with ({}x{}) at ({}x{})",
                    address,
                    scale(client.width, global.scale),
                    scale(client.height, global.scale),
                    scale(client.x - workspace.x as i16, global.scale) as f64,
                    scale(client.y - workspace.y as i16, global.scale) as f64
                );
                workspace_fixed.put(
                    &client_button,
                    scale(client.x - workspace.x as i16, global.scale) as f64,
                    scale(client.y - workspace.y as i16, global.scale) as f64,
                );
                monitor_data.client_refs.insert(*address, client_button);
            }
        }
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

fn click_workspace(button: &Button, workspace_id: WorkspaceId) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of workspace box");
        send_to_socket(&TransferType::Close(CloseConfig::Windows(
            WindowsOverride::WorkspaceID(workspace_id),
        )))
        .warn("unable send return to socket");
    });
}
