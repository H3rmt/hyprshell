use crate::components::shortcut_dialog::*;

use crate::util::{SetCursor, SetTextIfDifferent, mod_key_to_accelerator};
use config_lib::io;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct Switch {
    enabled: bool,
    config: io::Switch,
    prev_enabled: bool,
    prev_config: io::Switch,

    name: &'static str,
    keyboard_shortcut: Controller<KeyboardShortcut>,
}

#[derive(Debug)]
pub enum SwitchInput {
    Set(Option<io::Switch>),
    SetPrev(Option<io::Switch>),
    ResetSwitch,
    _OpenKeyboardShortcut,
    _Enabled(bool),
}

#[derive(Debug)]
pub enum SwitchOutput {
    Enabled(Option<io::Switch>),
    Key(String),
    Modifier(config_lib::Modifier),
    FilterSameClass(bool),
    FilterWorkspace(bool),
    FilterMonitor(bool),
    SwitchWorkspaces(bool),
    ExcludeWorkspaces(String),
    KillKey(Option<char>),
}

#[derive(Debug)]
pub struct SwitchInit {
    pub name: &'static str,
}

#[relm4::component(pub)]
impl SimpleComponent for Switch {
    type Init = SwitchInit;
    type Input = SwitchInput;
    type Output = SwitchOutput;

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
            })
            .connect_receiver(move |_, out| {
                #[allow(clippy::match_wildcard_for_single_variants)]
                match out {
                    KeyboardShortcutOutput::SetKey(r#mod, key) => {
                        outs.emit(SwitchOutput::Key(key));
                        outs.emit(SwitchOutput::Modifier(r#mod));
                    }
                    KeyboardShortcutOutput::OpenRequest => {
                        ins.emit(SwitchInput::_OpenKeyboardShortcut);
                    }
                    _ => {}
                }
            });

        let model = Self {
            enabled: false,
            prev_enabled: false,
            config: Default::default(),
            prev_config: Default::default(),

            name: init.name,
            keyboard_shortcut,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        adw::ExpanderRow {
            set_title_selectable: true,
            set_show_enable_switch: true,
            set_hexpand: true,
            set_css_classes: &["enable-frame"],
            add_prefix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Center,
                set_spacing: 25,
                gtk::Label {
                    set_label: model.name,
                },
                model.keyboard_shortcut.widget().clone() -> gtk::Button {
                    #[watch]
                    set_sensitive: model.enabled,
                },
                _adw::ShortcutLabel::new(&mod_key_to_accelerator(model.config.modifier, &model.config.key)) {
                    #[watch]
                    set_accelerator: &mod_key_to_accelerator(model.config.modifier, &model.config.key),
                    #[watch]
                    set_css_classes: if model.enabled {
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
            set_enable_expansion: model.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.input_sender().emit(SwitchInput::_Enabled(e.enables_expansion()));} @h,
            #[watch]
            set_expanded: model.enabled,
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_css_classes: &["frame-row"],
                set_spacing: 30,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        #[watch]
                        set_css_classes: if model.config.filter_by == model.prev_config.filter_by { &[] } else { &["blue-label"]  },
                        set_label: "Filter",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Filter the shown windows by the provided filters")
                    },
                    adw::ExpanderRow {
                        #[watch]
                        set_title: &format!("{:?}", model.config.filter_by),
                        set_hexpand: true,
                        set_title_lines: 2,
                        set_css_classes: &["item-expander"],
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_2)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::SameClass),
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(SwitchOutput::FilterSameClass(c.is_active())); } @h_2,
                            set_title: "Same class",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_3)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::CurrentWorkspace),
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(SwitchOutput::FilterWorkspace(c.is_active())); } @h_3,
                            set_title: "Current workspace",
                        },
                        add_row = &adw::SwitchRow {
                            #[watch]
                            #[block_signal(h_4)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::CurrentMonitor),
                            connect_active_notify[sender] => move |c| { sender.output_sender().emit(SwitchOutput::FilterMonitor(c.is_active())); } @h_4,
                            set_title: "Current monitor",
                        }
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Switch Workspaces",
                        #[watch]
                        set_css_classes: if model.config.switch_workspaces == model.prev_config.switch_workspaces { &[] } else { &["blue-label"] },
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Switch between workspaces in the Switch mode instead of windows")
                    },
                    gtk::Switch {
                        #[watch]
                        #[block_signal(h_5)]
                        set_active: model.config.switch_workspaces,
                        connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::SwitchWorkspaces(e.is_active())); } @h_5,
                        set_valign: gtk::Align::Center,
                    },
                },
            },
            add_row = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_css_classes: &["frame-row"],
                set_spacing: 30,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Exclude workspaces",
                        #[watch]
                        set_css_classes: if model.config.exclude_workspaces == model.prev_config.exclude_workspaces { &[] } else { &["blue-label"] },
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Exclude workspaces by regex \n(hyprctl workspaces -j | jq \".[].name\")")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("special:(htop)|irrelevant"),
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        #[block_signal(h_6)]
                        set_text_if_different: &model.config.exclude_workspaces,
                        connect_changed[sender] => move |e| { sender.output_sender().emit(SwitchOutput::ExcludeWorkspaces(e.text().into())) } @h_6,
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        set_label: "Key to kill window",
                        #[watch]
                        set_css_classes: if model.config.kill_key == '\0' { &["red-label"] } else if model.config.kill_key == model.prev_config.kill_key { &[] } else { &["blue-label"] },
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Press this key to kill the focused window")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("q"),
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        #[block_signal(h_7)]
                        set_text_if_different: &(if model.config.kill_key == '\0' { "".to_string() } else { model.config.kill_key.to_string() }),
                        connect_changed[sender] => move |e| { sender.output_sender().emit(SwitchOutput::KillKey(e.text().to_string().chars().next())) } @h_7,
                    }
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        tracing::trace!("switch::update: {message:?}");
        match message {
            SwitchInput::_Enabled(enabled) => match enabled {
                true => sender
                    .output_sender()
                    .emit(SwitchOutput::Enabled(Some(self.config.clone()))),
                false => sender.output_sender().emit(SwitchOutput::Enabled(None)),
            },
            SwitchInput::_OpenKeyboardShortcut => {
                // self.keyboard_shortcut
                //     .emit(KeyboardShortcutInput::ShowKeyboardShortcutDialog(
                //         Some((self.config.modifier, self.config.key.clone())),
                //         None,
                //     ));
            }
            SwitchInput::Set(config) => {
                match config {
                    Some(config) => {
                        self.config = config;
                        self.enabled = true;
                    }
                    None => {
                        self.enabled = false;
                    }
                };
            }
            SwitchInput::SetPrev(config) => {
                match config {
                    Some(config) => {
                        self.prev_config = config;
                        self.prev_enabled = true;
                    }
                    None => {
                        self.prev_enabled = false;
                    }
                };
            }
            SwitchInput::ResetSwitch => {
                self.config = self.prev_config.clone();
                self.enabled = self.prev_enabled;
            }
        }
    }
}
