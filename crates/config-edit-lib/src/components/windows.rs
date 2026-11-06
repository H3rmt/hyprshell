use crate::components::switch::*;

use config_lib::io;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct Windows {
    enabled: bool,
    config: io::Windows,
    prev_enabled: bool,
    prev_config: io::Windows,

    // overview: Controller<WindowsOverview>,
    switch: Controller<Switch>,
    switch_2: Controller<Switch>,
}

#[derive(Debug)]
pub enum WindowsInput {
    Set(Option<io::Windows>),
    SetPrev(Option<io::Windows>),
    Reset,
    _Enabled(bool),
}

#[derive(Debug)]
pub enum WindowsOutput {
    Enabled(Option<io::Windows>),
    Scale(f64),
    ItemsPerRow(u8),

    // Overview(WindowsOverviewOutput),
    Switch(SwitchOutput),
    Switch2(SwitchOutput),
}

#[derive(Debug)]
pub struct WindowsInit {}

#[relm4::component(pub)]
impl SimpleComponent for Windows {
    type Init = WindowsInit;
    type Input = WindowsInput;
    type Output = WindowsOutput;

    #[allow(clippy::cast_sign_loss)]
    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let switch = Switch::builder()
            .launch(SwitchInit { name: "Switch 1" })
            .forward(sender.output_sender(), WindowsOutput::Switch);
        let switch_2 = Switch::builder()
            .launch(SwitchInit { name: "Switch 2" })
            .forward(sender.output_sender(), WindowsOutput::Switch2);

        let model = Self {
            enabled: false,
            prev_enabled: false,
            config: Default::default(),
            prev_config: Default::default(),

            switch,
            switch_2,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 14,
            set_margin_top: 12,
            set_margin_bottom: 12,
            set_margin_start: 14,
            set_margin_end: 14,
            set_css_classes: &["page"],
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 16,
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    set_hexpand: true,
                    gtk::Label {
                        set_label: "Windows",
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["setting-title"],
                    },
                    gtk::Label {
                        set_label: "Configure the overview and switcher",
                        set_halign: gtk::Align::Start,
                        set_css_classes: &["setting-subtitle"],
                    },
                },
                gtk::Switch {
                    set_valign: gtk::Align::Center,
                    #[watch]
                    #[block_signal(h)]
                    set_active: model.enabled,
                    connect_active_notify[sender] => move |e| { sender.input_sender().emit(WindowsInput::_Enabled(e.is_active())); } @h,
                },
            },
            gtk::Frame {
                set_css_classes: &["setting-card"],
                gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    set_css_classes: &["boxed-list"],
                    // gtk::Box {
                    //     set_orientation: gtk::Orientation::Horizontal,
                    //     set_css_classes: &["setting-card-heading"],
                    //     gtk::Label {
                    //         set_label: "Display",
                    //         set_halign: gtk::Align::Start,
                    //         set_css_classes: &["setting-card-title"],
                    //     }
                    // },
                    adw::ActionRow {
                      set_subtitle: "Scaling (x x x)",
                      set_title: "Scale",
                      #[watch]
                      set_css_classes: if (model.config.scale - model.prev_config.scale).abs() < 0.01 { &[] } else { &["blue-label"] },
                      add_suffix = &gtk::SpinButton {
                          set_adjustment: &gtk::Adjustment::new(8.5, 0.5, 15.0, 0.5, 1.0, 0.0),
                          set_width_request: 220,
                          set_valign: gtk::Align::Center,
                          set_digits: 2,
                          #[watch]
                          #[block_signal(h_2)]
                          set_value: model.config.scale,
                          connect_value_changed[sender] => move |e| { sender.output_sender().emit(WindowsOutput::Scale((e.value() * 100.0).round() / 100.0)); } @h_2,
                      }
                    },
                    adw::ActionRow {
                      set_subtitle: "Test",
                      set_title: "Items per row",
                      #[watch]
                      set_css_classes: if model.config.items_per_row == model.prev_config.items_per_row { &[] } else { &["blue-label"] },
                      add_suffix = &gtk::SpinButton {
                          set_adjustment: &gtk::Adjustment::new(5.0, 1.0, 100.0, 1.0, 5.0, 0.0),
                          set_width_request: 220,
                          set_valign: gtk::Align::Center,
                          set_digits: 0,
                          #[watch]
                          #[block_signal(h_4)]
                          set_value: f64::from(model.config.items_per_row),
                          connect_value_changed[sender] => move |e| { sender.output_sender().emit(WindowsOutput::ItemsPerRow(e.value() as u8)) } @h_4,
                      }
                    },
                }
            },
            model.switch.widget(),
            model.switch_2.widget(),
        }
    }

    fn update(&mut self, message: WindowsInput, sender: ComponentSender<Self>) {
        tracing::trace!("windows::update: {message:?}");
        match message {
            WindowsInput::_Enabled(enabled) => match enabled {
                true => sender
                    .output_sender()
                    .emit(WindowsOutput::Enabled(Some(self.config.clone()))),
                false => sender.output_sender().emit(WindowsOutput::Enabled(None)),
            },
            WindowsInput::Set(config) => {
                match config {
                    Some(config) => {
                        self.config = config;
                        self.enabled = true;
                    }
                    None => {
                        self.enabled = false;
                    }
                };
                self.switch
                    .emit(SwitchInput::Set(self.config.switch.clone()));
                self.switch_2
                    .emit(SwitchInput::Set(self.config.switch_2.clone()));
            }
            WindowsInput::SetPrev(config) => {
                match config {
                    Some(config) => {
                        self.prev_config = config;
                        self.prev_enabled = true;
                    }
                    None => {
                        self.prev_enabled = false;
                    }
                };
                self.switch
                    .emit(SwitchInput::SetPrev(self.prev_config.switch.clone()));
                self.switch_2
                    .emit(SwitchInput::SetPrev(self.prev_config.switch_2.clone()));
            }
            WindowsInput::Reset => {
                self.config = self.prev_config.clone();
                self.enabled = self.prev_enabled;
                self.switch.emit(SwitchInput::ResetSwitch);
                self.switch_2.emit(SwitchInput::ResetSwitch);
            }
        }
    }
}
