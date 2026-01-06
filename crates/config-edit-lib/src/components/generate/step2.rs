use crate::util::{SelectRow, SetTextIfDifferent};
use core_lib::util::find_command;
use relm4::adw::prelude::*;
use relm4::gtk::{Align, Justification, SelectionMode};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use relm4::{adw, gtk};
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::trace;

#[derive(Debug)]
pub struct Step2 {
    list_box: gtk::ListBox,
    entry: adw::EntryRow,
}

#[derive(Debug)]
pub enum Step2Input {
    // external set method
    SetData(Option<String>),
    // internal set method
    ISetData(Option<String>),
}

#[derive(Debug)]
pub struct Step2Init {}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Step2 {
    type Init = Step2Init;
    type Input = Step2Input;
    type Output = Option<String>;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 40,
            gtk::Label::new(Some("Default Terminal")) {
                set_css_classes: &["title-1"],
                set_align: Align::Center,
                set_justify: Justification::Center,
            },
            #[local_ref]
            list_box -> gtk::ListBox {
                set_halign: Align::Center,
                set_valign: Align::Start,
                set_hexpand: true,
                set_selection_mode: SelectionMode::Single,
                set_css_classes: &["boxed-list", "generate-min-width"],
                connect_row_activated[sender] => move |_, row| {
                    if let Some(wdg) = row.downcast_ref::<adw::ActionRow>() {
                        let title = wdg.title().to_string();
                        trace!("press title: {title}");
                        match &*title {
                            "Autodetect" => {
                                sender.input(Step2Input::ISetData(None));
                            }
                            "Alacritty" => {
                                sender.input(Step2Input::ISetData(Some("alacritty".to_string())));
                            }
                            "Kitty" => {
                                sender.input(Step2Input::ISetData(Some("kitty".to_string())));
                            }
                            "Wezterm" => {
                                sender.input(Step2Input::ISetData(Some("wezterm".to_string())));
                            }
                            "Foot" => {
                                sender.input(Step2Input::ISetData(Some("foot".to_string())));
                            }
                            "QTerminal" => {
                                sender.input(Step2Input::ISetData(Some("qterminal".to_string())));
                            }
                            "Lilyterm" => {
                                sender.input(Step2Input::ISetData(Some("lilyterm".to_string())));
                            }
                            "Tilix" => {
                                sender.input(Step2Input::ISetData(Some("tilix".to_string())));
                            }
                            "Terminix" => {
                                sender.input(Step2Input::ISetData(Some("terminix".to_string())));
                            }
                            "Konsole" => {
                                sender.input(Step2Input::ISetData(Some("konsole".to_string())));
                            }
                            _ => {}
                        }
                    }
                },
                adw::ActionRow {
                    set_title: "Autodetect",
                    set_activatable: true,
                    set_subtitle: "autodetect from list of known terminals"
                },
                adw::ActionRow {
                    set_title: "Alacritty",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "alacritty") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "alacritty").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Kitty",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "kitty") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "kitty").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Wezterm",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "wezterm") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "wezterm").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Foot",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "foot") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "foot").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "QTerminal",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "qterminal") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "qterminal").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Lilyterm",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "lilyterm") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "lilyterm").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Tilix",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "tilix") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "tilix").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Terminix",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "terminix") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "terminix").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                adw::ActionRow {
                    set_title: "Konsole",
                    set_activatable: true,
                    set_css_classes: if get_terminals().iter().any(|(name, _)| *name == "konsole") {&[]} else {&["gray-label"]},
                    set_subtitle: &get_terminals().iter().find(|(name, _)| *name == "konsole").map(|(_, path)| path.display().to_string()).unwrap_or_default()
                },
                #[local_ref]
                entry -> adw::EntryRow {
                    set_title: "Custom",
                    connect_changed[sender] => move |e| { sender.input(Step2Input::ISetData(Some(e.text().into())))} @h_20,
                    set_input_purpose: gtk::InputPurpose::FreeForm,
                },
            },
            gtk::Label::new(Some("used to open terminal applications like htop")) {
                set_css_classes: &["title-4"],
                set_justify: Justification::Center,
                set_vexpand: true,
                set_valign: Align::End,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_box = gtk::ListBox::default();
        let entry = adw::EntryRow::default();
        let model = Self {
            list_box: list_box.clone(),
            entry: entry.clone(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher::step2:update: {message:?}");
        match message {
            Step2Input::ISetData(data) => {
                sender.input(Step2Input::SetData(data.clone()));
                sender.output_sender().emit(data);
            }
            Step2Input::SetData(data) => {
                self.list_box.select_row_index(match data.as_deref() {
                    None => 0,
                    Some("alacritty") => 1,
                    Some("kitty") => 2,
                    Some("wezterm") => 3,
                    Some("foot") => 4,
                    Some("qterminal") => 5,
                    Some("lilyterm") => 6,
                    Some("tilix") => 7,
                    Some("terminix") => 8,
                    Some("konsole") => 9,
                    _ => 10,
                });
                self.entry.set_text_if_different(
                    if data.is_some()
                        && data != Some("alacritty".to_string())
                        && data != Some("kitty".to_string())
                        && data != Some("wezterm".to_string())
                        && data != Some("foot".to_string())
                        && data != Some("qterminal".to_string())
                        && data != Some("lilyterm".to_string())
                        && data != Some("tilix".to_string())
                        && data != Some("terminix".to_string())
                        && data != Some("konsole".to_string())
                    {
                        data.as_ref().map_or("", |v| v)
                    } else {
                        ""
                    },
                );
            }
        }
    }
}

const TERMINALS: &[&str] = &[
    "alacritty",
    "kitty",
    "wezterm",
    "foot",
    "qterminal",
    "lilyterm",
    "tilix",
    "terminix",
    "konsole",
];

fn get_terminals() -> &'static Vec<(&'static str, PathBuf)> {
    static TERMS: OnceLock<Vec<(&'static str, PathBuf)>> = OnceLock::new();
    TERMS.get_or_init(|| {
        let mut out = vec![];
        for terminal in TERMINALS {
            trace!("terminal: {terminal}");
            if let Some(path) = find_command(terminal) {
                out.push((*terminal, path));
            }
        }
        out
    })
}
