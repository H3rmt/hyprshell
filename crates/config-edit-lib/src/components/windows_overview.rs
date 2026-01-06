use crate::components::shortcut_dialog::{
    KeyboardShortcut, KeyboardShortcutInit, KeyboardShortcutInput, KeyboardShortcutOutput,
};
use crate::flags_csv;
use crate::structs::ConfigModifier;
use crate::util::{SetCursor, SetTextIfDifferent, mod_key_to_accelerator};
use relm4::Component;
use relm4::adw::prelude::*;
use relm4::gtk::Align;
use relm4::{ComponentController, Controller};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug)]
pub struct WindowsOverview {
    config: crate::Overview,
    prev_config: crate::Overview,
    keyboard_shortcut: Controller<KeyboardShortcut>,
}

#[derive(Debug)]
pub enum WindowsOverviewInput {
    SetOverview(crate::Overview),
    SetPrevOverview(crate::Overview),
    ResetOverview,
    OpenKeyboardShortcut,
}

#[derive(Debug)]
pub struct WindowsOverviewInit {
    pub config: crate::Overview,
}

#[derive(Debug)]
pub enum WindowsOverviewOutput {
    Enabled(bool),
    Key(String),
    Modifier(ConfigModifier),
    FilterSameClass(bool),
    FilterWorkspace(bool),
    FilterMonitor(bool),
    ExcludeSpecialWorkspaces(String),
}

#[relm4::component(pub)]
impl SimpleComponent for WindowsOverview {
    type Init = WindowsOverviewInit;
    type Input = WindowsOverviewInput;
    type Output = WindowsOverviewOutput;

    view! {
        #[root]
        adw::ExpanderRow {
            set_title_selectable: true,
            set_show_enable_switch: true,
            set_hexpand: true,
            set_css_classes: &["enable-frame"],
            add_prefix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: Align::Fill,
                set_valign: Align::Center,
                set_spacing: 25,
                gtk::Label {
                    set_label: "Overview + Launcher",
                },
                model.keyboard_shortcut.widget().clone() -> gtk::Button {
                    #[watch]
                    set_sensitive: model.config.enabled,
                },
                _adw::ShortcutLabel::new(&mod_key_to_accelerator(model.config.modifier, &model.config.key)) {
                    #[watch]
                    set_accelerator: &mod_key_to_accelerator(model.config.modifier, &model.config.key),
                    #[watch]
                    set_css_classes: if model.config.enabled {
                        if mod_key_to_accelerator(model.config.modifier, &model.config.key) == mod_key_to_accelerator(model.prev_config.modifier, &model.prev_config.key)
                            { &[] }
                        else
                            { &["blue-label"] }
                    } else {
                        &["gray-label"]
                    },
                },
            },
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: model.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(WindowsOverviewOutput::Enabled(e.enables_expansion()));} @h,
            #[watch]
            set_expanded: model.config.enabled,
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_css_classes: &["frame-row"],
                set_spacing: 30,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        #[watch]
                        set_css_classes: if model.config.same_class == model.prev_config.same_class &&
                            model.config.current_workspace == model.prev_config.current_workspace &&
                            model.config.current_monitor == model.prev_config.current_monitor { &[] } else { &["blue-label"]  },
                        set_label: "Filter",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Filter the shown windows by the provided filters")
                    },
                    adw::ExpanderRow {
                        #[watch]
                        set_title: &flags_csv!(model.config,same_class,current_monitor,current_workspace),
                        set_hexpand: true,
                        set_title_lines: 2,
                        set_css_classes: &["item-expander"],
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_1)]
                            set_active: model.config.same_class,
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(WindowsOverviewOutput::FilterSameClass(c.is_active())); } @h_1,
                            set_title: "Same class",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_2)]
                            set_active: model.config.current_workspace,
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(WindowsOverviewOutput::FilterWorkspace(c.is_active())); } @h_2,
                            set_title: "Current workspace",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_3)]
                            set_active: model.config.current_monitor,
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(WindowsOverviewOutput::FilterMonitor(c.is_active())); } @h_3,
                            set_title: "Current monitor",
                        }
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Exclude special workspaces (TODO)",
                        #[watch]
                        set_css_classes: if model.config.exclude_special_workspaces == model.prev_config.exclude_special_workspaces { &[] } else { &["blue-label"] },
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Exclude special workspaces by regex \n(hyprctl workspaces -j | jq \".[].name\")")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("special:(monitor|second)"),
                        set_hexpand: true,
                        set_valign: Align::Center,
                        #[watch]
                        #[block_signal(h_4)]
                        set_text_if_different: &model.config.exclude_special_workspaces,
                        connect_changed[sender] => move |e| { sender.output_sender().emit(WindowsOverviewOutput::ExcludeSpecialWorkspaces(e.text().into())); } @h_4,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let outs = sender.output_sender().clone();
        let ins = sender.input_sender().clone();
        let keyboard_shortcut = KeyboardShortcut::builder()
            .launch(KeyboardShortcutInit {
                label: None,
                icon: Some("keyboard-layout".to_string()),
                init: Some((init.config.modifier, init.config.key.clone())),
            })
            .connect_receiver(move |_, out| match out {
                KeyboardShortcutOutput::SetKey(r#mod, key) => {
                    outs.emit(WindowsOverviewOutput::Key(key));
                    outs.emit(WindowsOverviewOutput::Modifier(r#mod));
                }
                KeyboardShortcutOutput::OpenRequest => {
                    ins.emit(WindowsOverviewInput::OpenKeyboardShortcut);
                }
                _ => {}
            });

        let model = Self {
            config: init.config.clone(),
            prev_config: init.config,
            keyboard_shortcut,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: WindowsOverviewInput, _sender: ComponentSender<Self>) {
        trace!("windows_overview::update: {message:?}");
        match message {
            WindowsOverviewInput::SetOverview(config) => {
                self.config = config;
            }
            WindowsOverviewInput::SetPrevOverview(config) => {
                self.prev_config = config;
            }
            WindowsOverviewInput::ResetOverview => {
                self.config = self.prev_config.clone();
            }
            WindowsOverviewInput::OpenKeyboardShortcut => {
                self.keyboard_shortcut
                    .emit(KeyboardShortcutInput::ShowKeyboardShortcutDialog(
                        Some((self.config.modifier, self.config.key.clone())),
                        None,
                    ));
            }
        }
    }
}
