use crate::LauncherData;
use crate::plugins::{DetailsMenuItem, get_sortable_launch_options, get_static_launch_options};
use async_channel::Sender;
use config_lib::Modifier;
use core_lib::transfer::{CloseOverviewConfig, Identifier, TransferType};
use core_lib::{WarnWithDetails, default};
use gtk::gdk::Cursor;
use gtk::pango::EllipsizeMode;
use gtk::prelude::*;
use gtk::{
    Align, Button, IconSize, Image, Label, ListBox, ListBoxRow, Orientation, Overflow, Popover,
    SelectionMode, glib,
};
use std::collections::HashMap;
use std::path::Path;
use tracing::{Level, debug, debug_span, span, warn};

pub fn update_launcher(data: &mut LauncherData, text: String, event_sender: Sender<TransferType>) {
    let _span = debug_span!("update_launcher").entered();

    while let Some(child) = data.results_box.first_child() {
        data.results_box.remove(&child);
    }
    while let Some(child) = data.plugins_box.first_child() {
        data.plugins_box.remove(&child);
    }
    data.sorted_matches.clear();
    data.static_matches.clear();
    if !data.config.show_when_empty && text.is_empty() {
        return;
    }

    let sortable_launch_options =
        get_sortable_launch_options(&data.config.plugins, &text, &data.config.data_dir);
    let mut items = data.config.max_items.min(9);
    for (index, opt) in sortable_launch_options.into_iter().enumerate() {
        if items == 0 {
            break;
        }
        items -= 1;
        let (row, details) = create_entry(
            &opt.iden,
            match index {
                0 => "Return".to_string(),
                i if i <= 9 => format!("{}+{i}", data.config.launch_modifier),
                _ => "".to_string(),
            },
            opt.icon,
            &opt.name,
            opt.details,
            opt.details_long,
            opt.details_menu,
            event_sender.clone(),
        );
        data.results_box.append(&row);
        data.results_items.insert(opt.iden.clone(), (row, details));
        data.sorted_matches.push(opt.iden);
    }

    let static_launch_options =
        get_static_launch_options(&data.config.plugins, &data.config.default_terminal);
    for opt in static_launch_options.into_iter() {
        let button = create_static_plugin_box(
            &opt.iden,
            opt.icon,
            &opt.text,
            &opt.details,
            opt.key,
            data.config.launch_modifier,
            event_sender.clone(),
        );
        data.plugins_box.append(&button);
        data.plugins_items.insert(opt.iden.clone(), button);
        data.static_matches.insert(opt.key, opt.iden);
    }
}

fn create_static_plugin_box(
    iden: &Identifier,
    icon: Option<Box<Path>>,
    text: &str,
    details: &str,
    key: char,
    launch_modifier: Modifier,
    event_sender: Sender<TransferType>,
) -> Button {
    let hbox = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["launcher-plugin-inner"])
        .spacing(6)
        .build();

    if let Some(icon) = icon {
        // trace!("icon: {:?}", icon);
        let icon = Image::builder()
            .icon_size(IconSize::Large)
            .icon_name(icon.to_string_lossy())
            .build();
        hbox.append(&icon);
    }

    let vbox = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();

    let title = Label::builder()
        .halign(Align::Center)
        .valign(Align::Start)
        .label(text)
        .css_classes(["underline"])
        .tooltip_text(details)
        .build();
    vbox.append(&title);

    let exec = Label::builder()
        .halign(Align::Center)
        .valign(Align::End)
        .ellipsize(EllipsizeMode::End)
        .css_classes(["launcher-key"])
        .label(format!("{launch_modifier} + {key}"))
        .build();
    vbox.append(&exec);

    hbox.append(&vbox);

    let button = Button::builder()
        .child(&hbox)
        .css_classes(["launcher-plugin"])
        .build();
    button.set_cursor(Cursor::from_name("pointer", None).as_ref());
    click_plugin(&button, iden.clone(), event_sender);
    button
}

#[allow(clippy::too_many_arguments)]
fn create_entry(
    iden: &Identifier,
    key: impl Into<glib::GString>,
    icon_path: Option<Box<Path>>,
    name: &str,
    details: Box<str>,
    details_long: Option<Box<str>>,
    details_menu: Vec<DetailsMenuItem>,
    event_sender: Sender<TransferType>,
) -> (gtk::Box, HashMap<Identifier, ListBoxRow>) {
    let hbox = gtk::Box::builder()
        .css_classes(["launcher-item-inner"])
        .orientation(Orientation::Horizontal)
        .height_request(45)
        .spacing(8)
        .hexpand(true)
        .vexpand(true)
        .build();

    let icon = Image::builder().icon_size(IconSize::Large).build();
    if let Some(icon_path) = icon_path {
        if icon_path.is_absolute() {
            if let Some(icon_name) = icon_path.file_stem() {
                if default::theme_has_icon_name(&icon_name.to_string_lossy()) {
                    icon.set_icon_name(Some(&icon_name.to_string_lossy()));
                } else {
                    icon.set_from_file(Some(Path::new(&*icon_path)));
                }
            } else {
                warn!("invalid icon name: {icon_path:?}");
            }
        } else {
            // use filename as some files are named org.gnome.file
            // trace!(
            //     "using name: {:?}",
            //     icon_path.file_name().and_then(|name| name.to_str())
            // );
            icon.set_icon_name(icon_path.file_name().and_then(|name| name.to_str()));
        }
    }
    hbox.append(&icon);

    let title = Label::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .label(name)
        .build();
    hbox.append(&title);

    let exec = Label::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .hexpand(true)
        .css_classes(["launcher-exec"])
        .ellipsize(EllipsizeMode::End)
        .label(details)
        .build();
    if let Some(details_long) = details_long {
        exec.set_tooltip_text(Some(&details_long));
        exec.add_css_class("underline")
    }
    hbox.append(&exec);

    let mut details_list = HashMap::new();
    if !details_menu.is_empty() {
        let button = Button::builder()
            .css_classes(["launcher-other-menu-button"])
            .icon_name("open-menu-symbolic")
            .halign(Align::End)
            .valign(Align::Center)
            .build();
        let menu = Popover::builder()
            .css_classes(["launcher-other-menu"])
            .has_arrow(false)
            .overflow(Overflow::Hidden)
            .build();
        let menu_list_box = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .build();

        for item in details_menu {
            let menu_item_text = Label::builder()
                .css_classes(["underline"])
                .label(format!("{}: {}", name, item.text))
                .build();
            let menu_item_button = Button::builder()
                .css_classes(["launcher-other-menu-item-inner"])
                .child(&menu_item_text)
                .tooltip_text(item.exec)
                .build();
            let menu_item = ListBoxRow::builder()
                .css_classes(["launcher-other-menu-item"])
                .child(&menu_item_button)
                .build();
            menu_item.set_cursor(Cursor::from_name("pointer", None).as_ref());
            click_details_entry(&menu_item_button, item.iden.clone(), event_sender.clone());
            menu_list_box.append(&menu_item);
            details_list.insert(item.iden.clone(), menu_item);
        }
        menu.set_parent(&button);
        menu.set_child(Some(&menu_list_box));
        button.connect_clicked(move |_button| {
            menu.popup();
        });
        hbox.append(&button);
    }

    let index_label = Label::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .css_classes(["launcher-key"])
        .label(key)
        .build();
    hbox.append(&index_label);

    let outer_box = gtk::Box::builder().css_classes(["launcher-item"]).build();
    outer_box.append(&hbox);

    outer_box.set_cursor(Cursor::from_name("pointer", None).as_ref());
    click_entry(&outer_box, iden.clone(), event_sender);
    (outer_box, details_list)
}

fn click_plugin(button: &Button, iden: Identifier, event_sender: Sender<TransferType>) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of launcher entry");
        event_sender
            .send_blocking(TransferType::CloseOverview(
                CloseOverviewConfig::LauncherClick(iden.clone()),
            ))
            .warn_details("unable to send");
    });
}

fn click_entry(button: &gtk::Box, iden: Identifier, event_sender: Sender<TransferType>) {
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        debug!("Exiting on click of launcher entry");
        event_sender
            .send_blocking(TransferType::CloseOverview(
                CloseOverviewConfig::LauncherClick(iden.clone()),
            ))
            .warn_details("unable to send");
    });
    button.add_controller(gesture);
}

fn click_details_entry(button: &Button, iden: Identifier, event_sender: Sender<TransferType>) {
    button.connect_clicked(move |_| {
        debug!("Exiting on click of launcher details entry");
        event_sender
            .send_blocking(TransferType::CloseOverview(
                CloseOverviewConfig::LauncherClick(iden.clone()),
            ))
            .warn_details("unable to send");
    });
}
