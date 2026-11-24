use crate::components::shortcut_dialog::{
    KeyboardShortcut, KeyboardShortcutInit, KeyboardShortcutInput, KeyboardShortcutOutput,
};
use crate::components::switch::{filter_summary, filters_match};
use crate::util::SetTextIfDifferent;
use config_lib::io;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct Overview {
    enabled: bool,
    config: io::Overview,
    prev_enabled: bool,
    prev_config: io::Overview,
    keyboard_shortcut: Controller<KeyboardShortcut>,
}

#[derive(Debug)]
pub enum OverviewInput {
    Set(Option<io::Overview>),
    SetPrev(Option<io::Overview>),
    Reset,
    Enabled(bool),
    OpenKeyboardShortcut,
}

#[derive(Debug)]
pub enum OverviewOutput {
    Enabled(Option<io::Overview>),
    Shortcut(io::Modifier, String),
    FilterSameClass(bool),
    FilterWorkspace(bool),
    FilterMonitor(bool),
    ExcludeWorkspaces(String),
    TopOffset(u16),
    LivePreview(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for Overview {
    type Init = ();
    type Input = OverviewInput;
    type Output = OverviewOutput;

    fn init(
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let keyboard_shortcut = KeyboardShortcut::builder()
            .launch(KeyboardShortcutInit {
                label: None,
                icon: Some("keyboard-layout".to_string()),
                require_modifier: false,
            })
            .connect_receiver({
                let sender = sender.clone();
                move |_, output| match output {
                    KeyboardShortcutOutput::SetKey(modifier, key) => {
                        sender
                            .output_sender()
                            .emit(OverviewOutput::Shortcut(modifier, key));
                    }
                    KeyboardShortcutOutput::OpenRequest => {
                        sender.input(OverviewInput::OpenKeyboardShortcut)
                    }
                    KeyboardShortcutOutput::Abort => (),
                }
            });
        let model = Self {
            enabled: false,
            config: Default::default(),
            prev_enabled: false,
            prev_config: Default::default(),
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
                        set_label: "Overview",
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: if model.enabled == model.prev_enabled { &["setting-title"] } else { &["setting-title", "blue-label"] },
                    },
                    gtk::Label {
                        set_label: "Configure the window overview",
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["setting-subtitle"],
                    },
                },
                gtk::Switch {
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(enabled_handler)]
                    set_active: model.enabled,
                    connect_active_notify[sender] => move |e| sender.input(OverviewInput::Enabled(e.is_active())) @enabled_handler,
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
                        set_subtitle: "Shortcut to open the overview",
                        #[watch]
                        set_css_classes: if model.config.key == model.prev_config.key && model.config.modifier == model.prev_config.modifier { &[] } else { &["blue-label"] },
                        add_suffix = &gtk::Box {
                            set_valign: gtk::Align::Center,
                            set_spacing: 16,
                            adw::ShortcutLabel::new("") {
                                set_valign: gtk::Align::Center,
                                #[watch]
                                set_visible: !crate::util::mod_key_to_accelerator(model.config.modifier, &model.config.key).is_empty(),
                                #[watch]
                                set_accelerator: &crate::util::mod_key_to_accelerator(model.config.modifier, &model.config.key),
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
                            #[block_signal(same_class_handler)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::SameClass),
                            connect_active_notify[sender] => move |e| sender.output_sender().emit(OverviewOutput::FilterSameClass(e.is_active())) @same_class_handler,
                        },
                        add_row = &adw::SwitchRow {
                            set_title: "Current workspace",
                            set_subtitle: "Cannot be combined with current monitor",
                            #[watch]
                            set_css_classes: if model.config.filter_by.contains(&io::FilterBy::CurrentWorkspace) == model.prev_config.filter_by.contains(&io::FilterBy::CurrentWorkspace) { &[] } else { &["blue-label"] },
                            #[watch]
                            #[block_signal(workspace_handler)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::CurrentWorkspace),
                            connect_active_notify[sender] => move |e| sender.output_sender().emit(OverviewOutput::FilterWorkspace(e.is_active())) @workspace_handler,
                        },
                        add_row = &adw::SwitchRow {
                            set_title: "Current monitor",
                            set_subtitle: "Cannot be combined with current workspace",
                            #[watch]
                            set_css_classes: if model.config.filter_by.contains(&io::FilterBy::CurrentMonitor) == model.prev_config.filter_by.contains(&io::FilterBy::CurrentMonitor) { &[] } else { &["blue-label"] },
                            #[watch]
                            #[block_signal(monitor_handler)]
                            set_active: model.config.filter_by.contains(&io::FilterBy::CurrentMonitor),
                            connect_active_notify[sender] => move |e| sender.output_sender().emit(OverviewOutput::FilterMonitor(e.is_active())) @monitor_handler,
                        },
                    },
                    adw::ActionRow {
                        set_title: "Live preview",
                        set_subtitle: "Show live window previews",
                        #[watch]
                        set_css_classes: if model.config.live_preview == model.prev_config.live_preview { &[] } else { &["blue-label"] },
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,
                            #[watch]
                            #[block_signal(preview_handler)]
                            set_active: model.config.live_preview,
                            connect_active_notify[sender] => move |e| sender.output_sender().emit(OverviewOutput::LivePreview(e.is_active())) @preview_handler,
                        },
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
                            #[block_signal(exclude_handler)]
                            set_text_if_different: &model.config.exclude_workspaces,
                            connect_changed[sender] => move |e| sender.output_sender().emit(OverviewOutput::ExcludeWorkspaces(e.text().into())) @exclude_handler,
                        },
                    },
                    adw::ActionRow {
                        set_title: "Top offset",
                        set_subtitle: "Distance from the top of the screen in pixels",
                        #[watch]
                        set_css_classes: if model.config.top_offset == model.prev_config.top_offset { &[] } else { &["blue-label"] },
                        add_suffix = &gtk::SpinButton {
                            set_adjustment: &gtk::Adjustment::new(0.0, 0.0, f64::from(u16::MAX), 1.0, 50.0, 0.0),
                            set_width_request: 220,
                            set_valign: gtk::Align::Center,
                            set_digits: 0,
                            #[watch]
                            #[block_signal(offset_handler)]
                            set_value: f64::from(model.config.top_offset),
                            connect_value_changed[sender] => move |e| sender.output_sender().emit(OverviewOutput::TopOffset(e.value() as u16)) @offset_handler,
                        },
                    },
                },
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            OverviewInput::OpenKeyboardShortcut => {
                self.keyboard_shortcut
                    .emit(KeyboardShortcutInput::ShowKeyboardShortcutDialog(
                        Some((self.config.modifier, self.config.key.to_string())),
                        None,
                    ));
            }
            OverviewInput::Enabled(enabled) => sender.output_sender().emit(
                OverviewOutput::Enabled(enabled.then(|| self.config.clone())),
            ),
            OverviewInput::Set(config) => {
                self.enabled = config.is_some();
                if let Some(config) = config {
                    self.config = config;
                }
            }
            OverviewInput::SetPrev(config) => {
                self.prev_enabled = config.is_some();
                if let Some(config) = config {
                    self.prev_config = config;
                }
            }
            OverviewInput::Reset => {
                self.enabled = self.prev_enabled;
                self.config = self.prev_config.clone();
            }
        }
    }
}
