use crate::WindowsOverviewData;
use async_channel::Sender;
use core_lib::transfer::{CloseOverviewConfig, ExternalTransferType, WindowsOverride};
use core_lib::{Active, ClientId, HyprlandData, WarnWithDetails};
use regex::Regex;
use relm4::adw::gdk::Cursor;
use relm4::adw::prelude::*;
use relm4::gtk;
use relm4::gtk::prelude::ApplicationWindowExt;
use relm4::gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use std::borrow::Cow;
use tracing::{debug, instrument, trace};

fn scale<T: Into<f64>>(value: T, scale: f64) -> i32 {
    (value.into() / (15f64 - scale)) as i32
}

#[instrument(level = "debug", skip_all)]
#[allow(clippy::too_many_lines)]
pub fn render_overview(
    data: &mut WindowsOverviewData,
    hypr_data: HyprlandData,
    active: Active,
    remove_html: &Regex,
    event_sender: &Sender<ExternalTransferType>,
) {
    for (window, monitor_data) in &mut data.window_list {
        trace!("Showing window {:?}", window.id());
        window.set_visible(true);

        'workspaces: for (wid, workspace) in &hypr_data.workspaces {
            if workspace.monitor != monitor_data.id {
                continue 'workspaces;
            }
            trace!(
                "Creating workspace {wid} with ({}x{})",
                scale(workspace.width, data.config.scale),
                scale(workspace.height, data.config.scale)
            );
            let workspace_fixed = Fixed::builder()
                .width_request(scale(workspace.width, data.config.scale))
                .height_request(scale(workspace.height, data.config.scale))
                .build();
            let id_string = wid.to_string();
            let title = if workspace.name.trim().is_empty() {
                Cow::from(&id_string)
            } else {
                remove_html.replace_all(&workspace.name, "")
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
                button
            };
            if *wid < 0 {
                workspace_button.add_css_class("special");
            }
            monitor_data.workspaces_flow.insert(&workspace_button, -1);
            monitor_data.workspaces.insert(*wid, workspace_button);

            'clients: for (address, client) in &hypr_data.clients {
                if client.workspace != *wid {
                    continue 'clients;
                }
                let client_button = {
                    let title = if client.title.trim().is_empty() {
                        &client.class
                    } else {
                        &client.title
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
                    if client_h_w > 60 {
                        let image = Image::builder()
                            .css_classes(["client-image"])
                            .pixel_size((f64::from(client_h_w.clamp(50, 600)) / 1.7) as i32 - 20)
                            .build();
                        if !client.enabled {
                            image.add_css_class("monochrome");
                        }
                        crate::icon::set_icon(&client.class, client.pid, &image);
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
                    "Creating Client {address} with ({}x{}) at ({}x{})",
                    scale(client.width, data.config.scale),
                    scale(client.height, data.config.scale),
                    f64::from(scale(client.x, data.config.scale)),
                    f64::from(scale(client.y, data.config.scale))
                );
                workspace_fixed.put(
                    &client_button,
                    f64::from(scale(client.x, data.config.scale)),
                    f64::from(scale(client.y, data.config.scale)),
                );
                monitor_data.clients.insert(*address, client_button);
            }
        }
    }

    data.active = active;
    data.initial_active = active;
    data.hypr_data = hypr_data;
}

fn click_client(button: &Button, client_id: ClientId, event_sender: Sender<ExternalTransferType>) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of client button");
        // event_sender
        //     .send_blocking(ExternalTransferType::CloseOverview(
        //         CloseOverviewConfig::Windows(WindowsOverride::ClientId(client_id)),
        //     ))
        //     .warn_details("unable to send");
    });
}
