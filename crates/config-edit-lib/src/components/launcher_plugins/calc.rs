use crate::util::{SetCursor, SetTextIfDifferent};
use relm4::adw::prelude::*;
use relm4::gtk::Align;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug)]
pub struct CalcPlugin {
    config: crate::CalcPluginConfig,
    prev_config: crate::CalcPluginConfig,
}

#[derive(Debug)]
pub enum CalcPluginInput {
    Set(crate::CalcPluginConfig),
    SetPrev(crate::CalcPluginConfig),
    Reset,
}

#[derive(Debug)]
pub struct CalcPluginInit {
    pub config: crate::CalcPluginConfig,
}

#[derive(Debug)]
pub enum CalcPluginOutput {
    Enabled(bool),
    SetPrefix(Option<String>),
}

#[relm4::component(pub)]
impl SimpleComponent for CalcPlugin {
    type Init = CalcPluginInit;
    type Input = CalcPluginInput;
    type Output = CalcPluginOutput;

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
                set_spacing: 15,
                gtk::Label {
                    set_label: "Calculator",
                },
                gtk::Image::from_icon_name("dialog-information-symbolic") {
                    set_cursor_by_name: "help",
                    set_tooltip_text: Some("Calculates any mathematical expression typed into the launcher.")
                },
            },
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: model.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(CalcPluginOutput::Enabled(e.enables_expansion()))} @h,
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
                        set_css_classes: if model.config.prefix == model.prev_config.prefix { &[] } else { &["blue-label"]  },
                        set_label: "Restrict to prefix",
                    },
                    gtk::Switch {
                        #[watch]
                        #[block_signal(h_4)]
                        set_active: model.config.prefix.is_some(),
                        set_valign: Align::Center,
                        connect_active_notify[sender] => move |e| { sender.output_sender().emit(CalcPluginOutput::SetPrefix(if e.is_active() { Some(String::from("=")) } else { None })); } @h_4,
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Only show calculation outputs if the input starts with the prefix. The Prefix will be stripped from the input before calculating.")
                    },
                    gtk::Entry {
                        set_input_purpose: gtk::InputPurpose::FreeForm,
                        set_placeholder_text: Some("="),
                        set_hexpand: true,
                        set_valign: Align::Center,
                        #[watch]
                        set_sensitive: model.config.prefix.is_some(),
                        #[watch]
                        #[block_signal(h_5)]
                        set_text_if_different: &model.config.prefix.as_ref().unwrap_or(&String::new()),
                        connect_changed[sender] => move |e| { sender.output_sender().emit(CalcPluginOutput::SetPrefix(Some(e.text().into())))} @h_5,
                    }
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            config: init.config.clone(),
            prev_config: init.config,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        trace!("launcher_plugins::simple::update: {message:?}");
        match message {
            CalcPluginInput::Set(config) => {
                self.config = config;
            }
            CalcPluginInput::SetPrev(config) => {
                self.prev_config = config;
            }
            CalcPluginInput::Reset => {
                self.config = self.prev_config.clone();
            }
        }
    }
}
