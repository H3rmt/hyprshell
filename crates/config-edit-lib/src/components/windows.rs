use crate::components::switch::*;

use crate::util::SetCursor;
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
        // let windows_overview = WindowsOverview::builder()
        //     .launch(WindowsOverviewInit {
        //         config: init.config.overview.clone(),
        //     })
        //     .forward(sender.output_sender(), WindowsOutput::Overview);
        let switch = Switch::builder()
            .launch(SwitchInit { name: "Switch" })
            .forward(sender.output_sender(), WindowsOutput::Switch);
        let switch_2 = Switch::builder()
            .launch(SwitchInit {
                name: "Switch 2 (TODO)",
            })
            .forward(sender.output_sender(), WindowsOutput::Switch2);

        let model = Self {
            enabled: false,
            prev_enabled: false,
            config: Default::default(),
            prev_config: Default::default(),

            // overview: windows_overview,
            switch,
            switch_2,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 10,
            adw::ExpanderRow {
                set_title_selectable: true,
                set_show_enable_switch: true,
                set_hexpand: true,
                set_css_classes: &[""],
                set_title: "Windows (Overview and Switch)",
                #[watch]
                #[block_signal(h)]
                set_enable_expansion: model.enabled,
                connect_enable_expansion_notify[sender] => move |e| {sender.input_sender().emit(WindowsInput::_Enabled(e.enables_expansion())); } @h,
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
                            set_css_classes: if (model.config.scale - model.prev_config.scale).abs() < 0.01 { &[] } else { &["blue-label"]  },
                            set_label: "Scale",
                        },
                        gtk::Image::from_icon_name("dialog-information-symbolic") {
                            set_cursor_by_name: "help",
                            set_tooltip_text: Some("The scale used to scale down the real dimension the windows displayed in the overview. \nCan be set from `0.5 < X > to 15.0`")
                        },
                        gtk::SpinButton {
                            set_adjustment: &gtk::Adjustment::new(1.0, 0.5, 15.0, 0.5, 1.0, 0.0),
                            set_hexpand: true,
                            set_digits: 2,
                            #[watch]
                            #[block_signal(h_2)]
                            set_value: model.config.scale,
                            connect_value_changed[sender] => move |e| { sender.output_sender().emit(WindowsOutput::Scale((e.value() * 100.0).round() / 100.0)); } @h_2,
                        }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Label {
                            #[watch]
                            set_css_classes: if model.config.items_per_row == model.prev_config.items_per_row { &[] } else { &["blue-label"] },
                            set_label: "Items per row",
                        },
                        gtk::Image::from_icon_name("dialog-information-symbolic") {
                            set_cursor_by_name: "help",
                            set_tooltip_text: Some("The number of workspaces or windows to show per row. \nIf you have 6 workspaces open and set this to 3, you will see 2 rows of 3 workspaces")
                        },
                        gtk::SpinButton {
                            set_adjustment: &gtk::Adjustment::new(1.0, 1.0, 100.0, 1.0, 5.0, 0.0),
                            set_hexpand: true,
                            set_digits: 0,
                            #[watch]
                            #[block_signal(h_3)]
                            set_value: f64::from(model.config.items_per_row),
                            connect_value_changed[sender] => move |e| { sender.output_sender().emit(WindowsOutput::ItemsPerRow(e.value() as u8)) } @h_3,
                        }
                    }
                },
                // add_row = model.overview.widget(),
                add_row = model.switch.widget(),
                add_row = model.switch_2.widget(),
            }
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
                // self.overview.emit(WindowsOverviewInput::SetOverview(
                //     self.config.overview.clone(),
                // ));
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
                // self.overview.emit(WindowsOverviewInput::SetPrevOverview(
                //     self.prev_config.overview.clone(),
                // ));
                self.switch
                    .emit(SwitchInput::SetPrev(self.prev_config.switch.clone()));
                self.switch_2
                    .emit(SwitchInput::SetPrev(self.prev_config.switch_2.clone()));
            }
            WindowsInput::Reset => {
                self.config = self.prev_config.clone();
                self.enabled = self.prev_enabled;
                // self.overview.emit(WindowsOverviewInput::ResetOverview);
                self.switch.emit(SwitchInput::ResetSwitch);
                self.switch_2.emit(SwitchInput::ResetSwitch);
            }
        }
    }
}
