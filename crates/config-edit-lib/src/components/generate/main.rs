use crate::components::generate::step0::{Step0, Step0Init, Step0Input};
use crate::components::generate::step1::{LauncherPlugins, Step1, Step1Init, Step1Input};
use crate::components::generate::step2::{Step2, Step2Init, Step2Input};
use crate::components::generate::step3::{SearchEngines, Step3, Step3Init, Step3Input};
use crate::components::generate::step4::{Step4, Step4Init, Step4Input};
use crate::structs::ConfigModifier;
use crate::util::{ScrollToPosition, default_config};
use config_lib::SearchEngine;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::{Align, Justification};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent,
};
use relm4::{adw, gtk};
use std::path::Path;
use tracing::trace;

#[derive(Debug)]
pub struct Generate {
    themes_carousel: adw::Carousel,

    step: usize,
    step_boxes: Vec<gtk::Box>,
    explain_label: gtk::Label,

    step0: Controller<Step0>,
    step0_data: Option<(ConfigModifier, String)>,
    step1: Controller<Step1>,
    step1_data: LauncherPlugins,
    step2: Controller<Step2>,
    step2_data: Option<String>,
    step3: Controller<Step3>,
    step3_data: SearchEngines,
    step4: Controller<Step4>,
    step4_data: Option<(ConfigModifier, String)>,
}

#[derive(Debug)]
pub enum GenerateInput {
    Start,
    Advance(usize),
    Back(usize),
    SetStep0(Option<(ConfigModifier, String)>),
    SetStep1(LauncherPlugins),
    SetStep2(Option<String>),
    SetStep3(SearchEngines),
    SetStep4(Option<(ConfigModifier, String)>),
}

#[derive(Debug)]
pub struct GenerateInit {
    pub system_data_dir: Box<Path>,
}

#[derive(Debug)]
pub enum GenerateOutput {
    Finish(crate::Config),
}

struct Out {
    overview: Option<(ConfigModifier, String)>,
    launcher: LauncherPlugins,
    default_terminal: Option<String>,
    launcher_websearch_plugin: SearchEngines,
    switch: Option<(ConfigModifier, String)>,
}

const MAX_STEP: usize = 5;

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Generate {
    type Init = GenerateInit;
    type Input = GenerateInput;
    type Output = GenerateOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            gtk::Box {
                #[local_ref]
                themes_carousel -> adw::Carousel {
                    set_visible: true,
                    #[local_ref]
                    step0 -> gtk::Box {},
                    #[local_ref]
                    step1 -> gtk::Box {},
                    #[local_ref]
                    step2 -> gtk::Box {},
                    #[local_ref]
                    step3 -> gtk::Box {},
                    #[local_ref]
                    step4 -> gtk::Box {},
                    #[local_ref]
                    step5 -> gtk::Box {},
                },
                set_expand: true,
                set_spacing: 20,
                set_margin_all: 20,
                #[transition = "SlideUpDown"]
                #[name="step0_stack"]
                match model.step {
                    0 => *model.step0.widget(),
                    1 => *model.step1.widget(),
                    2 => *model.step2.widget(),
                    3 => *model.step3.widget(),
                    4 => *model.step4.widget(),
                    5 => {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_spacing: 20,
                            #[local_ref]
                            explain_label -> gtk::Label {
                                set_css_classes: &[],
                                set_align: Align::Center,
                                set_justify: Justification::Left,
                            },
                        }
                    },
                    _ => gtk::Label::new(Some("INVALID GENERATE STEP")) {}
                },
            },
            adw::CarouselIndicatorDots {
                set_carousel: Some(themes_carousel),
            },
            gtk::Box {
                set_spacing: 25,
                set_halign: Align::Center,
                gtk::Button {
                    set_label: "Back",
                    set_css_classes: &["suggested-action", "pill"],
                    connect_clicked[sender] => move |_| sender.input(GenerateInput::Back(1)),
                },
                gtk::Button {
                    #[watch]
                    set_label: if model.step == MAX_STEP { "Finish" } else { "Next" },
                    set_css_classes: &["suggested-action", "pill"],
                    connect_clicked[sender] => move |_| sender.input(GenerateInput::Advance(1)),
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let path = init.system_data_dir.join("setup_preview");
        let step0 = Step0::builder()
            .launch(Step0Init {
                system_data_dir: path.clone().into_boxed_path(),
            })
            .forward(sender.input_sender(), GenerateInput::SetStep0);
        let step1 = Step1::builder()
            .launch(Step1Init {})
            .forward(sender.input_sender(), GenerateInput::SetStep1);
        let step2 = Step2::builder()
            .launch(Step2Init {})
            .forward(sender.input_sender(), GenerateInput::SetStep2);
        let step3 = Step3::builder()
            .launch(Step3Init {})
            .forward(sender.input_sender(), GenerateInput::SetStep3);
        let step4 = Step4::builder()
            .launch(Step4Init {
                system_data_dir: path.into_boxed_path(),
            })
            .forward(sender.input_sender(), GenerateInput::SetStep4);

        let themes_carousel = adw::Carousel::builder().build();
        let mut step_boxes = vec![];
        for _ in 0..=MAX_STEP {
            step_boxes.push(gtk::Box::builder().build());
        }

        let explain_label = gtk::Label::default();

        let model = Self {
            step: 0,
            step_boxes,
            step0,
            step0_data: None,
            step1,
            step1_data: LauncherPlugins::default(),
            step2,
            step2_data: None,
            step3,
            step3_data: SearchEngines::default(),
            step4,
            step4_data: None,
            explain_label,
            themes_carousel,
        };

        let themes_carousel = &model.themes_carousel;

        let step0 = &model.step_boxes[0];
        let step1 = &model.step_boxes[1];
        let step2 = &model.step_boxes[2];
        let step3 = &model.step_boxes[3];
        let step4 = &model.step_boxes[4];
        let step5 = &model.step_boxes[5];
        let explain_label = &model.explain_label;
        let widgets = view_output!();

        widgets.step0_stack.set_transition_duration(500);
        ComponentParts { model, widgets }
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("launcher::update: {message:?}");
        match message {
            GenerateInput::Advance(by) => {
                if self.step + by <= MAX_STEP {
                    self.step += by;
                    self.themes_carousel.scroll_to_pos(self.step, false);
                } else {
                    let conf = Out {
                        overview: self.step0_data.clone(),
                        launcher: self.step1_data,
                        default_terminal: self.step2_data.clone(),
                        launcher_websearch_plugin: self.step3_data,
                        switch: self.step4_data.clone(),
                    }
                    .into();
                    sender.output_sender().emit(GenerateOutput::Finish(conf));
                }
                match self.step {
                    // advanced to the launcher setup
                    1 => {
                        // skip launcher config if overview disabled
                        if self.step0_data.is_none() {
                            sender.input(GenerateInput::Advance(3));
                        }
                    }
                    // advanced to websearch plugin
                    3 => {
                        // skip websearch plugin config if websearch disabled
                        if !self.step1_data.search_the_web {
                            sender.input(GenerateInput::Advance(1));
                        }
                    }
                    5 => {
                        let conf: crate::Config = Out {
                            overview: self.step0_data.clone(),
                            launcher: self.step1_data,
                            default_terminal: self.step2_data.clone(),
                            launcher_websearch_plugin: self.step3_data,
                            switch: self.step4_data.clone(),
                        }
                        .into();
                        let conf: config_lib::Config = conf.into();
                        let explanation = config_lib::explain(&conf, None, false);
                        self.explain_label.set_label(&explanation);
                    }
                    _ => {}
                }
            }
            GenerateInput::Back(by) => {
                #[allow(clippy::cast_possible_wrap)]
                if (self.step as i64) - (by as i64) >= 0 {
                    self.step -= by;
                    self.themes_carousel.scroll_to_pos(self.step, false);
                }
                match self.step {
                    // stepped back to launcher setup
                    3 => {
                        // skip launcher config if overview disabled
                        if self.step0_data.is_none() {
                            sender.input(GenerateInput::Back(3));
                        }
                        // skip websearch plugin config if websearch disabled
                        if !self.step1_data.search_the_web {
                            sender.input(GenerateInput::Back(1));
                        }
                    }
                    _ => {}
                }
            }
            GenerateInput::Start => {
                self.step = 0;
                self.themes_carousel.scroll_to_pos(self.step, false);

                let default = crate::Config::from(default_config());
                self.step0_data = Some((
                    default.windows.overview.modifier,
                    default.windows.overview.key,
                ));
                self.step1_data = LauncherPlugins::default();
                self.step2_data = None;
                self.step3_data = SearchEngines::default();
                self.step4_data =
                    Some((default.windows.switch.modifier, default.windows.switch.key));
                self.step0
                    .emit(Step0Input::SetData(self.step0_data.clone()));
                self.step1.emit(Step1Input::SetData(self.step1_data));
                self.step2
                    .emit(Step2Input::SetData(self.step2_data.clone()));
                self.step3.emit(Step3Input::SetData(self.step3_data));
                self.step4
                    .emit(Step4Input::SetData(self.step4_data.clone()));
            }
            GenerateInput::SetStep0(data) => {
                self.step0_data = data;
            }
            GenerateInput::SetStep1(data) => {
                self.step1_data = data;
            }
            GenerateInput::SetStep2(step) => {
                self.step2_data = step;
            }
            GenerateInput::SetStep3(data) => {
                self.step3_data = data;
            }
            GenerateInput::SetStep4(data) => {
                self.step4_data = data;
            }
        }
    }
}

impl From<Out> for crate::Config {
    fn from(val: Out) -> Self {
        let mut config = Self::from(default_config());
        if let Some(overview) = &val.overview {
            config.windows.overview.enabled = true;
            config.windows.overview.modifier = overview.0;
            config.windows.overview.key.clone_from(&overview.1);

            config
                .windows
                .overview
                .launcher
                .plugins
                .applications
                .enabled = val.launcher.launch_applications;
            config.windows.overview.launcher.plugins.terminal.enabled =
                val.launcher.run_commands_in_terminal;
            config.windows.overview.launcher.plugins.shell.enabled =
                val.launcher.run_commands_in_background;
            config.windows.overview.launcher.plugins.websearch.enabled =
                val.launcher.search_the_web;
            config.windows.overview.launcher.plugins.calc.enabled =
                val.launcher.calculate_math_expressions;
            config.windows.overview.launcher.plugins.actions.enabled = val.launcher.run_actions;

            config.windows.overview.launcher.default_terminal = val.default_terminal;

            let mut vec = vec![];
            if val.launcher_websearch_plugin.google {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "Google")
                    .expect("engine not found: Google")
                    .1());
            }
            if val.launcher_websearch_plugin.startpage {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "Startpage")
                    .expect("engine not found: Startpage")
                    .1());
            }
            if val.launcher_websearch_plugin.duckduckgo {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "DuckDuckGo")
                    .expect("engine not found: DuckDuckGo")
                    .1());
            }
            if val.launcher_websearch_plugin.bing {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "Bing")
                    .expect("engine not found: Bing")
                    .1());
            }
            if val.launcher_websearch_plugin.wikipedia {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "Wikipedia")
                    .expect("engine not found: Wikipedia")
                    .1());
            }
            if val.launcher_websearch_plugin.chatgpt {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "ChatGpt")
                    .expect("engine not found: ChatGpt")
                    .1());
            }
            if val.launcher_websearch_plugin.youtube {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "YouTube")
                    .expect("engine not found: YouTube")
                    .1());
            }
            if val.launcher_websearch_plugin.reddit {
                vec.push(WEB_SEARCH_ENGINES
                    .iter()
                    .find(|(n, _)| *n == "Reddit")
                    .expect("engine not found: Reddit")
                    .1());
            }
            config.windows.overview.launcher.plugins.websearch.engines = vec;
        } else {
            config.windows.overview.enabled = false;
        }

        if let Some(switch) = &val.switch {
            config.windows.switch.enabled = true;
            config.windows.switch.modifier = switch.0;
            config.windows.switch.key.clone_from(&switch.1);
        } else {
            config.windows.switch.enabled = false;
        }
        config
    }
}

#[allow(clippy::type_complexity)]
pub const WEB_SEARCH_ENGINES: &[(&str, fn() -> SearchEngine)] = &[
    ("Google", || SearchEngine {
        url: "https://www.google.com/search?q={}".into(),
        name: "Google".into(),
        key: 'g',
    }),
    ("Startpage", || SearchEngine {
        url: "https://www.startpage.com/sp/search?query={}".into(),
        name: "Startpage".into(),
        key: 's',
    }),
    ("DuckDuckGo", || SearchEngine {
        url: "https://duckduckgo.com/?q={}".into(),
        name: "DuckDuckGo".into(),
        key: 'd',
    }),
    ("Bing", || SearchEngine {
        url: "https://www.bing.com/search?q={}".into(),
        name: "Bing".into(),
        key: 'b',
    }),
    ("Wikipedia", || SearchEngine {
        url: "https://en.wikipedia.org/wiki/Special:Search?search={}".into(),
        name: "Wikipedia".into(),
        key: 'w',
    }),
    ("ChatGpt", || SearchEngine {
        url: "https://chatgpt.com/?q={}".into(),
        name: "ChatGpt".into(),
        key: 'c',
    }),
    ("YouTube", || SearchEngine {
        url: "https://www.youtube.com/results?search_query={}".into(),
        name: "YouTube".into(),
        key: 'y',
    }),
    ("Reddit", || SearchEngine {
        url: "https://www.reddit.com/search?q={}".into(),
        name: "Reddit".into(),
        key: 'r',
    }),
];
