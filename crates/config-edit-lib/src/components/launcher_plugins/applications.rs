use crate::util::SetCursor;
use relm4::adw::gtk::{Adjustment, Align};
use relm4::adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug)]
pub struct Applications {
    config: crate::ApplicationsPluginConfig,
    prev_config: crate::ApplicationsPluginConfig,
}

#[derive(Debug)]
pub enum ApplicationsInput {
    Set(crate::ApplicationsPluginConfig),
    SetPrev(crate::ApplicationsPluginConfig),
    Reset,
}

#[derive(Debug)]
pub struct ApplicationsInit {
    pub config: crate::ApplicationsPluginConfig,
}

#[derive(Debug)]
pub enum ApplicationsOutput {
    Enabled(bool),
    ShowExecs(bool),
    ShowActions(bool),
    CacheWeeks(u8),
}

#[relm4::component(pub)]
impl SimpleComponent for Applications {
    type Init = ApplicationsInit;
    type Input = ApplicationsInput;
    type Output = ApplicationsOutput;

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
                    set_label: "Applications",
                },
                gtk::Image::from_icon_name("dialog-information-symbolic") {
                    set_cursor_by_name: "help",
                    set_tooltip_text: Some("Launch .desktop files, sorted based on recent usage")
                },
            },
            #[watch]
            #[block_signal(h)]
            set_enable_expansion: model.config.enabled,
            connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(ApplicationsOutput::Enabled(e.enables_expansion()));} @h,
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
                        set_css_classes: if model.config.run_cache_weeks == model.prev_config.run_cache_weeks { &[] } else { &["blue-label"]  },
                        set_label: "Run cache period (weeks)",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Number of weeks to retain run history; used to rank applications by usage.")
                    },
                    gtk::SpinButton {
                        set_adjustment: &Adjustment::new(0.0, 0.0, 52.0, 1.0, 8.0, 0.0),
                        set_digits: 0,
                        set_hexpand: true,
                        #[watch]
                        #[block_signal(h_1)]
                        set_value: f64::from(model.config.run_cache_weeks),
                        connect_value_changed[sender] => move |e| { sender.output_sender().emit(ApplicationsOutput::CacheWeeks(e.value() as u8)); } @h_1,
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        #[watch]
                        set_css_classes: if model.config.show_execs == model.prev_config.show_execs { &[] } else { &["blue-label"]  },
                        set_label: "Show Execs",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Show the exec line from the Desktop file. In the case of Flatpaks and PWAs these get shortened to the name of the app.")
                    },
                    gtk::Switch {
                        set_valign: Align::Center,
                        #[watch]
                        #[block_signal(h_2)]
                        set_active: model.config.show_execs,
                        connect_active_notify[sender] => move |e| { sender.output_sender().emit(ApplicationsOutput::ShowExecs(e.is_active())) } @h_2,
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    gtk::Label {
                        #[watch]
                        set_css_classes: if model.config.show_actions_submenu == model.prev_config.show_actions_submenu { &[] } else { &["blue-label"]  },
                        set_label: "Show Actions",
                    },
                    gtk::Image::from_icon_name("dialog-information-symbolic") {
                        set_cursor_by_name: "help",
                        set_tooltip_text: Some("Show a dropdown menu with all the desktop actions specified in the `.desktop` files of the applications, like `new private window`, etc.")
                    },
                    gtk::Switch {
                        set_valign: Align::Center,
                        #[watch]
                        #[block_signal(h_3)]
                        set_active: model.config.show_actions_submenu,
                        connect_active_notify[sender] => move |e| { sender.output_sender().emit(ApplicationsOutput::ShowActions(e.is_active())); } @h_3,
                    },
                },
            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
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
        trace!("launcher_plugins::application::update: {message:?}");
        match message {
            ApplicationsInput::Set(config) => {
                self.config = config;
            }
            ApplicationsInput::SetPrev(config) => {
                self.prev_config = config;
            }
            ApplicationsInput::Reset => {
                self.config = self.prev_config.clone();
            }
        }
    }
}
