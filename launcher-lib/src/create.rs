use crate::global::LauncherGlobalData;
use crate::r#match::get_matches;
use crate::LauncherGlobal;
use core_lib::theme_icon_cache::theme_has_icon_name;
use core_lib::transfer::{Override, ReturnConfig, TransferType};
use core_lib::{send_to_socket, Warn, LAUNCHER_NAMESPACE};
use gtk::gdk::Key;
use gtk::glib::{clone, Propagation};
use gtk::pango::EllipsizeMode;
use gtk::prelude::*;
use gtk::{glib, EventSequenceState, GestureClick, IconSize, Image, ListBoxRow};
use gtk::{
    Align, Application, ApplicationWindow, Entry, EventControllerKey, Label, ListBox, Orientation,
    SelectionMode,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use tracing::{debug, span, trace, warn, Level};

pub fn create_launcher_window(
    app: &Application,
    global: &mut LauncherGlobal,
    data_dir: &Path,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "create_launcher_window").entered();

    let main_vbox = ListBox::builder()
        .css_classes(vec!["launcher"])
        .width_request(global.width as i32)
        .selection_mode(SelectionMode::None)
        .build();

    let results = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(vec!["launcher-results"])
        .build();

    let max = global.max_items as usize;
    let show_execs = global.show_execs;
    let run_cache_weeks = global.run_cache_weeks;
    let show_shell = global.show_shell;
    let data_dir = PathBuf::from(data_dir);
    let entry = Entry::builder().css_classes(vec!["launcher-input"]).build();
    entry.connect_changed(clone!(
        #[weak]
        results,
        move |e| {
            trace!("Entry changed: {}", e.text());
            update(
                &e.text(),
                &results,
                max,
                run_cache_weeks,
                show_execs,
                show_shell,
                &data_dir,
            );
        }
    ));
    let controller = EventControllerKey::new();
    controller.connect_key_pressed(move |_, k, _, m| match (k, m) {
        (Key::Tab, _) => Propagation::Stop,
        (Key::ISO_Left_Tab, _) => Propagation::Stop,
        _ => Propagation::Proceed,
    });
    entry.add_controller(controller);
    main_vbox.append(&entry);
    main_vbox.append(&results);

    let window = ApplicationWindow::builder()
        .css_classes(vec!["window"])
        .application(app)
        .child(&main_vbox)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_namespace(Some(LAUNCHER_NAMESPACE));
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 20);
    window.present();
    window.set_visible(false);

    debug!("Created launcher window ({})", window.id());
    global.data = Some(RefCell::new(LauncherGlobalData {
        window,
        entry,
        results,
    }));

    Ok(())
}

fn update(
    text: &str,
    list: &ListBox,
    launcher_max_items: usize,
    run_cache_weeks: u8,
    show_launcher_execs: bool,
    show_shell: bool,
    data_dir: &Path,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    if text.is_empty() {
        return;
    }

    let matches = get_matches(
        text,
        launcher_max_items,
        run_cache_weeks,
        show_shell,
        data_dir,
    );
    for (index, (_, entry)) in matches.into_iter().take(launcher_max_items).enumerate() {
        let row = create_entry(
            index,
            entry.icon,
            &entry.name,
            if show_launcher_execs {
                Some(get_exec_label(&entry.exec))
            } else {
                None
            },
        );
        row.add_controller(click_entry(index as u8));
        list.append(&row);
    }
}

fn create_entry(
    index: usize,
    icon_path: Option<Box<Path>>,
    name: &str,
    exec: Option<String>,
) -> ListBoxRow {
    let hbox = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .vexpand(true)
        .build();

    let icon = Image::builder()
        .css_classes(vec!["launcher-icon"])
        .icon_size(IconSize::Large)
        .build();
    if let Some(icon_path) = icon_path {
        if icon_path.is_absolute() {
            if let Some(icon_name) = icon_path.file_stem() {
                if !theme_has_icon_name(&icon_name.to_string_lossy()) {
                    icon.set_from_file(Some(Path::new(&*icon_path)));
                } else {
                    icon.set_icon_name(Some(&icon_name.to_string_lossy()));
                }
            } else {
                warn!("invalid icon name: {icon_path:?}");
            }
        } else {
            // use filename as some files are named org.gnome.file
            trace!("using name: {:?}", icon_path.file_name().and_then(|name| name.to_str()));
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

    if let Some(exec) = exec {
        let exec = Label::builder()
            .halign(Align::Start)
            .valign(Align::Center)
            .hexpand(true)
            .css_classes(vec!["launcher-exec"])
            .ellipsize(EllipsizeMode::End)
            .label(exec)
            .build();
        hbox.append(&exec);
    } else {
        title.set_hexpand(true);
    }

    if let Some(label) = match index {
        0 => Some("Return".to_string()),
        i if i <= 9 => Some(format!("Ctrl+{}", i)),
        _ => None,
    } {
        let index_label = Label::builder()
            .halign(Align::End)
            .valign(Align::Center)
            .label(label)
            .build();
        hbox.append(&index_label);
    }

    let row = ListBoxRow::builder()
        .css_classes(vec!["launcher-item"])
        .height_request(45)
        .hexpand(true)
        .vexpand(true)
        .child(&hbox)
        .build();
    row
}

fn click_entry(offset: u8) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(move |gesture, _, _, _| {
        gesture.set_state(EventSequenceState::Claimed);
        debug!("Exiting on click of launcher entry");
        send_to_socket(&TransferType::Return(ReturnConfig {
            r#override: Some(Override::Offset(offset)),
        }))
        .warn("unable send return to socket");
    });
    gesture
}

fn get_exec_label(exec: &str) -> String {
    let exec_trim = exec.replace("'", "").replace("\"", "");
    // pwa detection
    if exec.contains("--app-id=") && exec.contains("--profile-directory=") {
        // "flatpak 'run'" = pwa from browser inside flatpak
        if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
            format!(
                "[Flatpak + PWA] {}",
                exec_trim
                    .split(' ')
                    .find(|s| s.contains("--command="))
                    .and_then(|s| s
                        .split('=')
                        .next_back()
                        .and_then(|s| s.split('/').next_back()))
                    .unwrap_or_default()
            )
        } else {
            // normal PWA
            format!(
                "[PWA] {}",
                exec.split(' ')
                    .next()
                    .and_then(|s| s.split('/').next_back())
                    .unwrap_or_default()
            )
        }
        // flatpak detection
    } else if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
        format!(
            "[Flatpak] {}",
            exec_trim
                .split(' ')
                .find(|s| s.contains("--command="))
                .and_then(|s| s
                    .split('=')
                    .next_back()
                    .and_then(|s| s.split('/').next_back()))
                .unwrap_or_default()
        )
    } else {
        // show only last part of /path/to/exec if it begins with /
        // otherwise show full exec
        if exec_trim.starts_with('/') {
            format!(
                "{}",
                exec_trim
                    .rsplit('/')
                    .find(|s| !s.is_empty())
                    .unwrap_or_default()
            )
        } else {
            format!("{}", exec_trim)
        }
    }
}
