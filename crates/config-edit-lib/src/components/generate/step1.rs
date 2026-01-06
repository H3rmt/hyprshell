use relm4::adw::prelude::*;
use relm4::gtk::{Align, Justification, SelectionMode};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use relm4::{adw, gtk};
use tracing::trace;

#[derive(Debug, Copy, Clone)]
pub struct LauncherPlugins {
    pub launch_applications: bool,
    pub run_commands_in_background: bool,
    pub run_commands_in_terminal: bool,
    pub search_the_web: bool,
    pub calculate_math_expressions: bool,
    pub run_actions: bool,
}

impl Default for LauncherPlugins {
    fn default() -> Self {
        Self {
            launch_applications: true,
            run_commands_in_background: false,
            run_commands_in_terminal: true,
            search_the_web: false,
            calculate_math_expressions: true,
            run_actions: true,
        }
    }
}

#[derive(Debug)]
pub struct Step1 {
    launch_apps: adw::SwitchRow,
    run_commands_in_background: adw::SwitchRow,
    run_commands_in_terminal: adw::SwitchRow,
    search_the_web: adw::SwitchRow,
    calculate_math_expressions: adw::SwitchRow,
    run_actions: adw::SwitchRow,
}

#[derive(Debug)]
pub enum Step1Input {
    // external set method
    SetData(LauncherPlugins),
    // internal set method
    _Update,
}

#[derive(Debug)]
pub struct Step1Init {}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Step1 {
    type Init = Step1Init;
    type Input = Step1Input;
    type Output = LauncherPlugins;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 40,
            gtk::Label::new(Some("Launcher Plugins")) {
                set_css_classes: &["title-1"],
                set_align: Align::Center,
                set_justify: Justification::Center,
            },
            gtk::ListBox {
                set_halign: Align::Center,
                set_valign: Align::Start,
                set_hexpand: true,
                set_selection_mode: SelectionMode::None,
                set_css_classes: &["items-list", "boxed-list", "generate-min-width"],
                #[local_ref]
                launch_apps -> adw::SwitchRow {
                    set_title: "Launch Applications",
                    set_subtitle: "Launch .desktop files, sorted based on recent usage",
                    connect_active_notify => Step1Input::_Update,
                },
                #[local_ref]
                run_commands_in_background -> adw::SwitchRow {
                    set_title: "Run Commands in Background",
                    set_subtitle: "Run shell commands without opening a terminal",
                    connect_active_notify => Step1Input::_Update,
                },
                #[local_ref]
                run_commands_in_terminal -> adw::SwitchRow {
                    set_title: "Run Commands in Terminal",
                    set_subtitle: "Run command in a new terminal",
                    connect_active_notify => Step1Input::_Update,
                },
                #[local_ref]
                search_the_web -> adw::SwitchRow {
                    set_title: "Search the Web",
                    set_subtitle: "Open browser searching for text with user defined search engine",
                    connect_active_notify => Step1Input::_Update,
                },
                #[local_ref]
                calculate_math_expressions -> adw::SwitchRow {
                    set_title: "Calculate Math expressions",
                    set_subtitle: "Calculate valid Expressions using Rink",
                    connect_active_notify => Step1Input::_Update,
                },
                #[local_ref]
                run_actions -> adw::SwitchRow {
                    set_title: "Run Actions (shutdown, sleep, custom, etc.)",
                    set_subtitle: "Run predefined / user defined actions",
                    connect_active_notify => Step1Input::_Update,
                },
            },
            gtk::Label::new(Some("")) {
                set_css_classes: &["title-4"],
                set_justify: Justification::Center,
                set_vexpand: true,
                set_valign: Align::End,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let launch_apps = adw::SwitchRow::default();
        let run_commands_in_background = adw::SwitchRow::default();
        let run_commands_in_terminal = adw::SwitchRow::default();
        let search_the_web = adw::SwitchRow::default();
        let calculate_math_expressions = adw::SwitchRow::default();
        let run_actions = adw::SwitchRow::default();

        let model = Self {
            launch_apps: launch_apps.clone(),
            run_commands_in_background: run_commands_in_background.clone(),
            run_commands_in_terminal: run_commands_in_terminal.clone(),
            search_the_web: search_the_web.clone(),
            calculate_math_expressions: calculate_math_expressions.clone(),
            run_actions: run_actions.clone(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher::step1:update: {message:?}");
        match message {
            Step1Input::_Update => {
                let data = self.get_data();
                sender.output_sender().emit(data);
            }
            Step1Input::SetData(data) => {
                self.launch_apps.set_active(data.launch_applications);
                self.run_commands_in_background
                    .set_active(data.run_commands_in_background);
                self.run_commands_in_terminal
                    .set_active(data.run_commands_in_terminal);
                self.search_the_web.set_active(data.search_the_web);
                self.calculate_math_expressions
                    .set_active(data.calculate_math_expressions);
                self.run_actions.set_active(data.run_actions);
            }
        }
    }
}

impl Step1 {
    fn get_data(&self) -> LauncherPlugins {
        LauncherPlugins {
            launch_applications: self.launch_apps.is_active(),
            run_commands_in_background: self.run_commands_in_background.is_active(),
            run_commands_in_terminal: self.run_commands_in_terminal.is_active(),
            search_the_web: self.search_the_web.is_active(),
            calculate_math_expressions: self.calculate_math_expressions.is_active(),
            run_actions: self.run_actions.is_active(),
        }
    }
}
