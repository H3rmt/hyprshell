use crate::components::shortcut_dialog::*;

use crate::util::SetTextIfDifferent;
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
    Shortcut(config_lib::io::Modifier, String),
    FilterSameClass(bool),
    FilterWorkspace(bool),
    FilterMonitor(bool),
    SwitchWorkspaces(bool),
    LivePreview(bool),
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
                require_modifier: true,
            })
            .connect_receiver(move |_, out| {
                #[allow(clippy::match_wildcard_for_single_variants)]
                match out {
                    KeyboardShortcutOutput::SetKey(r#mod, key) => {
                        outs.emit(SwitchOutput::Shortcut(r#mod, key));
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
      gtk::Box {
        set_orientation: gtk::Orientation::Vertical,
        set_spacing: 14,
        set_margin_top: 12,
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 16,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,
                set_hexpand: true,
                gtk::Label {
                    set_label: model.name,
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_css_classes: if model.enabled == model.prev_enabled { &["setting-title"] } else { &["setting-title", "blue-label"] },
                },
                gtk::Label {
                    set_label: "abbbbbbbbbbbbbbbbbbbbbbbbb",
                    set_halign: gtk::Align::Start,
                    set_css_classes: &["setting-subtitle"],
                },
            },
            gtk::Switch {
                set_valign: gtk::Align::Center,
                #[watch]
                #[block_signal(h)]
                set_active: model.enabled,
                connect_active_notify[sender] => move |e| { sender.input_sender().emit(SwitchInput::_Enabled(e.is_active())); } @h,
            },
        },
      gtk::Frame {
          set_css_classes: &["setting-card"],
          #[watch]
          set_visible: model.enabled,
          gtk::ListBox {
              set_selection_mode: gtk::SelectionMode::None,
              set_css_classes: &["boxed-list"],
              adw::ActionRow {
                set_title: "Keyboard shortcut",
                set_subtitle: "Shortcut to open the switcher",
                #[watch]
                set_css_classes: if model.config.key == model.prev_config.key && model.config.modifier == model.prev_config.modifier { &[] } else { &["blue-label"] },
                add_suffix = &gtk::Box {
                    set_valign: gtk::Align::Center,
                    set_spacing: 16,
                    gtk::Box {
                        set_spacing: 6,
                        set_valign: gtk::Align::Center,
                        adw::ShortcutLabel::new("") {
                            #[watch]
                            set_visible: !crate::util::is_keycode(&model.config.key),
                            #[watch]
                            set_accelerator: &crate::util::mod_key_to_accelerator(model.config.modifier, &model.config.key),
                        },
                        gtk::Label {
                            set_css_classes: &["keycap", "shortcut-code-keycap"],
                            #[watch]
                            set_visible: crate::util::is_keycode(&model.config.key) && model.config.modifier != io::Modifier::None,
                            #[watch]
                            set_label: crate::util::modifier_label(model.config.modifier),
                        },
                        gtk::Label {
                            set_css_classes: &["keycap", "shortcut-code-keycap"],
                            set_tooltip_text: Some("Hyprland physical keycode"),
                            #[watch]
                            set_visible: crate::util::is_keycode(&model.config.key),
                            #[watch]
                            set_label: &model.config.key,
                        },
                    },
                    model.keyboard_shortcut.widget().clone() -> gtk::Button {
                        set_valign: gtk::Align::Center,
                    },
                },
              },
              adw::ExpanderRow {
                set_title: "Filter windows",
                #[watch]
                set_subtitle: &filter_summary(&model.config.filter_by),
                #[watch]
                set_css_classes: if filters_match(&model.config.filter_by, &model.prev_config.filter_by) { &["expander"] } else { &["expander", "blue-label"] },
                add_row = &adw::SwitchRow {
                    set_title: "Same class",
                    #[watch]
                    set_css_classes: if model.config.filter_by.contains(&io::FilterBy::SameClass) == model.prev_config.filter_by.contains(&io::FilterBy::SameClass) { &[] } else { &["blue-label"] },
                    #[watch]
                    #[block_signal(h_2)]
                    set_active: model.config.filter_by.contains(&io::FilterBy::SameClass),
                    connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::FilterSameClass(e.is_active())); } @h_2,
                },
                add_row = &adw::SwitchRow {
                    set_title: "Current workspace",
                    set_subtitle: "Cannot be combined with current monitor",
                    #[watch]
                    set_css_classes: if model.config.filter_by.contains(&io::FilterBy::CurrentWorkspace) == model.prev_config.filter_by.contains(&io::FilterBy::CurrentWorkspace) { &[] } else { &["blue-label"] },
                    #[watch]
                    #[block_signal(h_3)]
                    set_active: model.config.filter_by.contains(&io::FilterBy::CurrentWorkspace),
                    connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::FilterWorkspace(e.is_active())); } @h_3,
                },
                add_row = &adw::SwitchRow {
                    set_title: "Current monitor",
                    set_subtitle: "Cannot be combined with current workspace",
                    #[watch]
                    set_css_classes: if model.config.filter_by.contains(&io::FilterBy::CurrentMonitor) == model.prev_config.filter_by.contains(&io::FilterBy::CurrentMonitor) { &[] } else { &["blue-label"] },
                    #[watch]
                    #[block_signal(h_4)]
                    set_active: model.config.filter_by.contains(&io::FilterBy::CurrentMonitor),
                    connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::FilterMonitor(e.is_active())); } @h_4,
                },
              },
              adw::ActionRow {
                set_subtitle: "Switch between workspaces instead of individual windows",
                set_title: "Switch workspaces",
                #[watch]
                set_css_classes: if model.config.switch_workspaces == model.prev_config.switch_workspaces { &[] } else { &["blue-label"] },
                add_suffix = &gtk::Switch {
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(h_5)]
                    set_active: model.config.switch_workspaces,
                    connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::SwitchWorkspaces(e.is_active())); } @h_5,
                }
              },
              adw::ActionRow {
                set_title: "Live preview",
                set_subtitle: "Show live window previews",
                #[watch]
                set_css_classes: if model.config.live_preview == model.prev_config.live_preview { &[] } else { &["blue-label"] },
                add_suffix = &gtk::Switch {
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(h_8)]
                    set_active: model.config.live_preview,
                    connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::LivePreview(e.is_active())); } @h_8,
                }
              },
              adw::ActionRow {
                set_title: "Excluded workspaces",
                set_subtitle: "Regular expression matching workspace names",
                #[watch]
                set_css_classes: if model.config.exclude_workspaces == model.prev_config.exclude_workspaces { &[] } else { &["blue-label"] },
                add_suffix = &gtk::Entry {
                    set_placeholder_text: Some("special:(htop)|irrelevant"),
                    set_width_request: 220,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(h_6)]
                    set_text_if_different: &model.config.exclude_workspaces,
                    connect_changed[sender] => move |e| { sender.output_sender().emit(SwitchOutput::ExcludeWorkspaces(e.text().into())); } @h_6,
                }
              },
              adw::ActionRow {
                set_title: "Key to close a window",
                set_subtitle: "A single character",
                #[watch]
                set_css_classes: if model.config.kill_key == '\0' { &["red-label"] } else if model.config.kill_key == model.prev_config.kill_key { &[] } else { &["blue-label"] },
                add_suffix = &gtk::Entry {
                    set_placeholder_text: Some("q"),
                    set_max_length: 1,
                    set_width_chars: 3,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(h_7)]
                    set_text_if_different: &(if model.config.kill_key == '\0' { String::new() } else { model.config.kill_key.to_string() }),
                    connect_changed[sender] => move |e| { sender.output_sender().emit(SwitchOutput::KillKey(e.text().chars().next())); } @h_7,
                }
              },
          }
        },
      },

    /*
            #[root]
            adw::ExpanderRow {
                set_title_selectable: true,
                set_show_enable_switch: true,
                set_hexpand: true,
                set_css_classes: &["enable-frame"],
                set_title: model.name,
                set_expanded: model.name == "Switch 2",
                add_row = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,
                    set_css_classes: &["switch-setting-row"],
                    gtk::Label {
                        set_label: "Shortcut",
                        set_width_request: 240,
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["windows-setting-label"],
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
                    gtk::Box {
                        set_hexpand: true,
                    },
                    model.keyboard_shortcut.widget().clone() -> gtk::Button {
                        set_label: "Change",
                        set_css_classes: &["rounded"],
                        #[watch]
                        set_sensitive: model.enabled,
                    },
                },
                #[watch]
                #[block_signal(h)]
                set_enable_expansion: model.enabled,
                connect_enable_expansion_notify[sender] => move |e| {sender.input_sender().emit(SwitchInput::_Enabled(e.enables_expansion()));} @h,
                add_row = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,
                    set_css_classes: &["switch-setting-row"],
                    gtk::Label {
                        set_label: "Filter",
                        set_width_request: 240,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: if model.config.filter_by == model.prev_config.filter_by { &["windows-setting-label"] } else { &["windows-setting-label", "blue-label"] },
                    },
                    adw::ExpanderRow {
                        #[watch]
                        set_title: &filter_summary(&model.config.filter_by),
                        set_hexpand: true,
                        set_title_lines: 1,
                        set_css_classes: &["item-expander", "filter-control"],
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
                    },
                },
                add_row = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_css_classes: &["switch-setting-row"],
                    gtk::Label {
                        set_label: "Switch workspaces",
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: if model.config.switch_workspaces == model.prev_config.switch_workspaces { &["windows-setting-label"] } else { &["windows-setting-label", "blue-label"] },
                    },
                    gtk::Switch {
                        #[watch]
                        #[block_signal(h_5)]
                        set_active: model.config.switch_workspaces,
                        connect_active_notify[sender] => move |e| { sender.output_sender().emit(SwitchOutput::SwitchWorkspaces(e.is_active())); } @h_5,
                        set_valign: gtk::Align::Center,
                    },
                },
                add_row = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_css_classes: &["switch-setting-row"],
                    gtk::Label {
                        set_label: "Excluded workspaces",
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: if model.config.exclude_workspaces == model.prev_config.exclude_workspaces { &["windows-setting-label"] } else { &["windows-setting-label", "blue-label"] },
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
                    },
                },
                add_row = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_css_classes: &["switch-setting-row"],
                    gtk::Label {
                        set_label: "Key to close a window",
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: if model.config.kill_key == '\0' { &["windows-setting-label", "red-label"] } else if model.config.kill_key == model.prev_config.kill_key { &["windows-setting-label"] } else { &["windows-setting-label", "blue-label"] },
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("Q"),
                        set_width_request: 56,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        #[block_signal(h_7)]
                        set_text_if_different: &(if model.config.kill_key == '\0' { String::new() } else { model.config.kill_key.to_string() }),
                        connect_changed[sender] => move |e| { sender.output_sender().emit(SwitchOutput::KillKey(e.text().to_string().chars().next())) } @h_7,
                    },
                }
            }
     */
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
                self.keyboard_shortcut
                    .emit(KeyboardShortcutInput::ShowKeyboardShortcutDialog(
                        Some((self.config.modifier, self.config.key.to_string())),
                        None,
                    ));
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

pub(super) fn filter_summary(filters: &[io::FilterBy]) -> String {
    if filters.is_empty() {
        return "No filters".to_string();
    }

    filters
        .iter()
        .map(|filter| match filter {
            io::FilterBy::SameClass => "Same class",
            io::FilterBy::CurrentWorkspace => "Current workspace",
            io::FilterBy::CurrentMonitor => "Current monitor",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn filters_match(filters: &[io::FilterBy], previous: &[io::FilterBy]) -> bool {
    [
        io::FilterBy::SameClass,
        io::FilterBy::CurrentWorkspace,
        io::FilterBy::CurrentMonitor,
    ]
    .iter()
    .all(|filter| filters.contains(filter) == previous.contains(filter))
}
