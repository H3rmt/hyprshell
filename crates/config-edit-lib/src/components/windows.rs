use crate::components::switch::{Switch, SwitchOutput};
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
use relm4::prelude::FactoryVecDeque;
use tracing::trace;

#[derive(Debug)]
pub struct Windows {
    pub overview: Controller<WindowsOverview>,
    pub config: crate::Windows,
    pub prev_config: crate::Windows,
    pub switches: FactoryVecDeque<Switch>,
}

#[derive(Debug)]
pub enum WindowsInput {
    Set(crate::Windows),
    SetPrev(crate::Windows),
    Reset,
    Switch(SwitchOutput),
    AddSwitch,
}

#[derive(Debug)]
pub enum WindowsOutput {
    Enabled(bool),
    Scale(f64),
    ItemsPerRow(u8),
    Overview(WindowsOverviewOutput),
    Switches(Vec<crate::Switch>),
}

#[derive(Debug)]
pub struct WindowsInit {
    pub config: crate::Windows,
}

#[allow(unused_assignments)]
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
                #[local_ref]
                add_row = switches -> gtk::ListBox {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_expand: true,
                    set_selection_mode: gtk::SelectionMode::None,
                    set_css_classes: &["items-list", "boxed-list"],
                },
                add_row = &adw::ButtonRow {
                    set_title: "Add switch profile",
                    connect_activated[sender] => move |_b| {
                        sender.input(WindowsInput::AddSwitch);
                    }
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
        let windows_overview = WindowsOverview::builder()
            .launch(WindowsOverviewInit {
                config: init.config.overview.clone(),
            })
            .forward(sender.output_sender(), WindowsOutput::Overview);
        let mut switches = FactoryVecDeque::builder()
            .launch(gtk::ListBox::builder().build())
            .forward(sender.input_sender(), WindowsInput::Switch);
        {
            let mut list = switches.guard();
            for switch in &init.config.switches {
                list.push_back(switch.clone());
            }
        }

        let model = Self {
            overview: windows_overview,
            switches,
            config: init.config.clone(),
            prev_config: init.config,
        };

        let switches = model.switches.widget();
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
                self.sync_switches();
            }
            WindowsInput::SetPrev(config) => {
                self.prev_config = config;
                self.overview.emit(WindowsOverviewInput::SetPrevOverview(
                    self.prev_config.overview.clone(),
                ));
            }
            WindowsInput::Reset => {
                self.config = self.prev_config.clone();
                self.overview.emit(WindowsOverviewInput::ResetOverview);
                self.sync_switches();
            }
            WindowsInput::Switch(msg) => match msg {
                SwitchOutput::Update(index, switch) => {
                    let idx = index.current_index();
                    if let Some(current) = self.config.switches.get_mut(idx) {
                        *current = switch;
                        _sender.output_sender().emit(WindowsOutput::Switches(self.config.switches.clone()));
                    }
                }
                SwitchOutput::Delete(index) => {
                    let idx = index.current_index();
                    if idx < self.config.switches.len() {
                        self.config.switches.remove(idx);
                        self.sync_switches();
                        _sender.output_sender().emit(WindowsOutput::Switches(self.config.switches.clone()));
                    }
                }
            },
            WindowsInput::AddSwitch => {
                self.config.switches.push(crate::Switch::default());
                self.sync_switches();
                _sender.output_sender().emit(WindowsOutput::Switches(self.config.switches.clone()));
            }
        }
    }
}

impl Windows {
    fn sync_switches(&mut self) {
        let mut list = self.switches.guard();
        let current_len = list.len();
        let new_len = self.config.switches.len();
        let shared = current_len.min(new_len);
        for idx in 0..shared {
            if let Some(item) = list.get_mut(idx) {
                item.update_config(self.config.switches[idx].clone());
            }
        }
        if new_len > current_len {
            for switch in self.config.switches.iter().skip(current_len) {
                list.push_back(switch.clone());
            }
        } else if new_len < current_len {
            for _ in new_len..current_len {
                list.pop_back();
            }
        }
    }
}
