use crate::components::switch::{Switch, SwitchInit, SwitchInput, SwitchOutput};
use crate::components::windows_overview::{
    WindowsOverview, WindowsOverviewInit, WindowsOverviewInput, WindowsOverviewOutput,
};
use crate::util::SetCursor;
use relm4::ComponentController;
use relm4::adw::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, Controller, RelmWidgetExt, SimpleComponent,
};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug)]
pub struct Windows {
    pub overview: Controller<WindowsOverview>,
    pub config: crate::Windows,
    pub prev_config: crate::Windows,
    pub switch: Controller<Switch>,
    pub switch_2: Controller<Switch>,
}

#[derive(Debug)]
pub enum WindowsInput {
    Set(crate::Windows),
    SetPrev(crate::Windows),
    Reset,
}

#[derive(Debug)]
pub enum WindowsOutput {
    Enabled(bool),
    Scale(f64),
    ItemsPerRow(u8),
    Overview(WindowsOverviewOutput),
    Switch(SwitchOutput),
    Switch2(SwitchOutput),
}

#[derive(Debug)]
pub struct WindowsInit {
    pub config: crate::Windows,
}

#[relm4::component(pub)]
impl SimpleComponent for Windows {
    type Init = WindowsInit;
    type Input = WindowsInput;
    type Output = WindowsOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 10,
            adw::ExpanderRow {
                set_title_selectable: true,
                set_show_enable_switch: true,
                set_hexpand: true,
                set_css_classes: &["enable-frame"],
                set_title: "Windows (Overview and Switch)",
                #[watch]
                #[block_signal(h)]
                set_enable_expansion: model.config.enabled,
                connect_enable_expansion_notify[sender] => move |e| {sender.output_sender().emit(WindowsOutput::Enabled(e.enables_expansion())); } @h,
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
                            set_adjustment: &gtk::Adjustment::new(1.0, 0.0, 50.0, 1.0, 5.0, 0.0),
                            set_hexpand: true,
                            set_digits: 0,
                            #[watch]
                            #[block_signal(h_3)]
                            set_value: f64::from(model.config.items_per_row),
                            connect_value_changed[sender] => move |e| { sender.output_sender().emit(WindowsOutput::ItemsPerRow(e.value() as u8)) } @h_3,
                        }
                    }
                },
                add_row = model.overview.widget(),
                add_row = model.switch.widget(),
                add_row = model.switch_2.widget(),
            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let windows_overview = WindowsOverview::builder()
            .launch(WindowsOverviewInit {
                config: init.config.overview.clone(),
            })
            .forward(sender.output_sender(), WindowsOutput::Overview);
        let switch = Switch::builder()
            .launch(SwitchInit {
                config: init.config.switch.clone(),
                name: "Switch",
            })
            .forward(sender.output_sender(), WindowsOutput::Switch);
        let switch_2 = Switch::builder()
            .launch(SwitchInit {
                config: init.config.switch_2.clone(),
                name: "Switch 2 (TODO)",
            })
            .forward(sender.output_sender(), WindowsOutput::Switch2);

        let model = Self {
            overview: windows_overview,
            switch,
            switch_2,
            config: init.config.clone(),
            prev_config: init.config,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: WindowsInput, _sender: ComponentSender<Self>) {
        trace!("windows::update: {message:?}");
        match message {
            WindowsInput::Set(config) => {
                self.config = config;
                self.overview.emit(WindowsOverviewInput::SetOverview(
                    self.config.overview.clone(),
                ));
                self.switch
                    .emit(SwitchInput::SetSwitch(self.config.switch.clone()));
                self.switch_2
                    .emit(SwitchInput::SetSwitch(self.config.switch_2.clone()));
            }
            WindowsInput::SetPrev(config) => {
                self.prev_config = config;
                self.overview.emit(WindowsOverviewInput::SetPrevOverview(
                    self.prev_config.overview.clone(),
                ));
                self.switch
                    .emit(SwitchInput::SetPrevSwitch(self.prev_config.switch.clone()));
                self.switch_2.emit(SwitchInput::SetPrevSwitch(
                    self.prev_config.switch_2.clone(),
                ));
            }
            WindowsInput::Reset => {
                self.config = self.prev_config.clone();
                self.overview.emit(WindowsOverviewInput::ResetOverview);
                self.switch.emit(SwitchInput::ResetSwitch);
                self.switch_2.emit(SwitchInput::ResetSwitch);
            }
        }
    }
}
