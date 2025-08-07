use crate::data::{SortConfig, collect_data};
use crate::global::WindowsSwitchData;
use crate::icon::set_icon;
use crate::next::find_next;
use anyhow::Context;
use core_lib::transfer::{Direction, OpenSwitch};
use core_lib::{ClientData, ClientId, WarnWithDetails};
use exec_lib::{get_current_monitor, set_remain_focused};
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use std::borrow::Cow;
use tracing::{Level, debug_span, span, trace};

fn scale(value: i16, scale: f64) -> i32 {
    (value as f64 / (15f64 - scale)) as i32
}

pub fn switch_already_open(data: &WindowsSwitchData) -> bool {
    data.window.get_visible()
}

pub fn open_switch(data: &mut WindowsSwitchData, config: OpenSwitch) -> anyhow::Result<()> {
    let _span = debug_span!("open_switch").entered();
    set_remain_focused().warn_details("Failed to set no follow mouse");

    let (clients_data, active_prev) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: true,
    })
    .context("Failed to collect data")?;
    let active = find_next(
        &if config.reverse {
            Direction::Left
        } else {
            Direction::Right
        },
        data.config.show_workspaces,
        true,
        &clients_data,
        active_prev,
        data.config.items_per_row as usize,
    );
    let remove_html = regex::Regex::new(r"<[^>]*>").context("Invalid regex")?;

    trace!("Showing window {:?}", data.window.id());
    data.window.set_visible(true);

    let current_monitor = get_current_monitor().context("Failed to get current monitor")?;

    if data.config.show_workspaces {
        for (wid, workspace) in &clients_data.workspaces {
            let clients: Vec<&(ClientId, ClientData)> = {
                let mut clients = clients_data
                    .clients
                    .iter()
                    .filter(|(_, client)| client.workspace == *wid && client.enabled)
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
            if clients.is_empty() {
                continue;
            }
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
                let button = Button::builder()
                    .child(&workspace_overlay)
                    .css_classes(["workspace", "no-hover"])
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());
                if active.workspace == *wid {
                    button.add_css_class("active");
                }
                button
            };
            data.main_flow.insert(&workspace_button, -1);
            data.workspaces.insert(*wid, workspace_button);

            for (address, client) in clients {
                if !client.enabled {
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
                        .css_classes(["client", "no-hover"])
                        .width_request(scale(client.width, data.config.scale))
                        .height_request(scale(client.height, data.config.scale))
                        .build();
                    button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                    // add initial border around initial active client
                    if active.client == Some(*address) {
                        button.add_css_class("active");
                    }
                    button
                };
                workspace_fixed.put(
                    &client_button,
                    scale(client.x - workspace.x as i16, data.config.scale) as f64,
                    scale(client.y - workspace.y as i16, data.config.scale) as f64,
                );
                data.clients.insert(*address, client_button);
            }
        }
    } else {
        for (address, client) in &clients_data.clients {
            if !client.enabled {
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
                    scale(current_monitor.height as i16, data.config.scale).min(scale(
                        (current_monitor.height as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ));
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
                    .css_classes(["client", "no-hover"])
                    .width_request(scale(
                        (current_monitor.width as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ))
                    .height_request(scale(
                        (current_monitor.height as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ))
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                // add border around initial active client
                if active.client == Some(*address) {
                    button.add_css_class("active");
                }
                button
            };
            data.main_flow.insert(&client_button, -1);
            data.clients.insert(*address, client_button);
        }
    }

    data.active = active;
    data.hypr_data = clients_data;
    Ok(())
}
