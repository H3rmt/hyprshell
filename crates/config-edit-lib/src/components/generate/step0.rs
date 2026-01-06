use crate::components::shortcut_dialog::{
    KeyboardShortcut, KeyboardShortcutInit, KeyboardShortcutInput, KeyboardShortcutOutput,
};
use crate::structs::ConfigModifier;
use crate::util::{SelectRow, mod_key_to_string};
use relm4::adw::gtk::{Align, Justification};
use relm4::adw::prelude::*;
use relm4::gtk::{SelectionMode, gio};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, WidgetRef,
};
use relm4::{adw, gtk};
use std::path::Path;
use tracing::trace;

#[derive(Debug)]
pub struct Step0 {
    keyboard_shortcut: Controller<KeyboardShortcut>,
    list_box: gtk::ListBox,
    button: adw::ButtonRow,
}

#[derive(Debug)]
pub enum Step0Input {
    // external set method
    SetData(Option<(ConfigModifier, String)>),
    // internal set method
    ISetData(Option<(ConfigModifier, String)>),
    OpenKeyboardShortcut(gtk::Widget),
}

#[derive(Debug)]
pub struct Step0Init {
    pub system_data_dir: Box<Path>,
}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Step0 {
    type Init = Step0Init;
    type Input = Step0Input;
    type Output = Option<(ConfigModifier, String)>;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 20,
            gtk::Label::new(Some("Key combination to open the overview and launcher")) {
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
                            "Disabled" => {
                                sender.input(Step0Input::ISetData(None));
                            }
                            "Super" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Super, "Super_L".to_string()))));
                            }
                            "Super + Tab" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Super, "Tab".to_string()))));
                            }
                            "Ctrl" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Ctrl, "Ctrl_L".to_string()))));
                            }
                            "Ctrl + Tab" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Ctrl, "Tab".to_string()))));
                            }
                            "Alt" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Alt, "Alt_L".to_string()))));
                            }
                            "Alt + Tab" => {
                                sender.input(Step0Input::ISetData(Some((ConfigModifier::Alt, "Tab".to_string()))));
                            }
                            _ => {}
                        }
                    }
                },
                adw::ActionRow {
                    set_title: "Disabled",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Super",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Super + Tab",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Ctrl",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Ctrl + Tab",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Alt",
                    set_activatable: true,
                },
                adw::ActionRow {
                    set_title: "Alt + Tab",
                    set_activatable: true,
                },
                #[local_ref]
                button -> adw::ButtonRow {
                    connect_activated[sender] => move |b| {
                        trace!("Generate: step0_keyboard_button toggled");
                        sender.input(Step0Input::OpenKeyboardShortcut(b.widget_ref().clone()));
                    }
                },
            },
            gtk::Picture {
                set_file: Some(&gio::File::for_path(init.system_data_dir.join("00_overview.png"))),
                set_css_classes: &["theme-image"],
                set_vexpand: true,
                set_hexpand: false,
                set_valign: Align::Fill,
                set_halign: Align::Center,
            },
            gtk::Label::new(Some("similar to gnome's overview\nShows all apps in a miniature view, allows to switch using arrow keys or tab.")) {
                set_css_classes: &["title-4"],
                set_justify: Justification::Center,
                set_vexpand: true,
                set_valign: Align::End,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ins = sender.input_sender().clone();
        let keyboard_shortcut = KeyboardShortcut::builder()
            .launch(KeyboardShortcutInit {
                label: Some("Custom".to_string()),
                icon: None,
                init: None,
            })
            .connect_receiver(move |_send, out| match out {
                KeyboardShortcutOutput::SetKey(r#mod, key) => {
                    // updates the label
                    ins.emit(Step0Input::ISetData(Some((r#mod, key))));
                }
                _ => {}
            });

        let list_box = gtk::ListBox::default();
        let button = adw::ButtonRow::default();
        let model = Self {
            keyboard_shortcut,
            button: button.clone(),
            list_box: list_box.clone(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher::step0:update: {message:?}");
        match message {
            Step0Input::ISetData(data) => {
                sender.input(Step0Input::SetData(data.clone()));
                sender.output_sender().emit(data);
            }
            Step0Input::SetData(data) => {
                self.list_box
                    .select_row_index(match data.as_ref().map(|(a, b)| (a, b.as_str())) {
                        None => 0,
                        Some((ConfigModifier::Super, "Super_L")) => 1,
                        Some((ConfigModifier::Super, "Tab")) => 2,
                        Some((ConfigModifier::Ctrl, "Ctrl_L")) => 3,
                        Some((ConfigModifier::Ctrl, "Tab")) => 4,
                        Some((ConfigModifier::Alt, "Alt_L")) => 5,
                        Some((ConfigModifier::Alt, "Tab")) => 6,
                        _ => 7,
                    });
                self.button.set_title(&format!(
                    "Custom: {}",
                    if data.is_some()
                        && data != Some((ConfigModifier::Super, "Super_L".to_string()))
                        && data != Some((ConfigModifier::Super, "Tab".to_string()))
                        && data != Some((ConfigModifier::Ctrl, "Ctrl_L".to_string()))
                        && data != Some((ConfigModifier::Ctrl, "Tab".to_string()))
                        && data != Some((ConfigModifier::Alt, "Alt_L".to_string()))
                        && data != Some((ConfigModifier::Alt, "Tab".to_string()))
                    {
                        data.as_ref()
                            .map(|(r#mod, key)| mod_key_to_string(*r#mod, key))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                ));
            }
            Step0Input::OpenKeyboardShortcut(widget) => {
                self.keyboard_shortcut
                    .emit(KeyboardShortcutInput::ShowKeyboardShortcutDialog(
                        None,
                        Some(widget),
                    ));
            }
        }
    }
}
