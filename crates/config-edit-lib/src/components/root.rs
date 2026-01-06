use crate::components::changes::{
    Changes, ChangesInit, ChangesInput, ChangesOutput, generate_items,
};
use crate::components::footer::{Footer, FooterInit, FooterInput, FooterOutput};
use crate::components::generate::{Generate, GenerateInit, GenerateInput, GenerateOutput};
use crate::components::launcher::{Launcher, LauncherInit, LauncherInput, LauncherOutput};
use crate::components::launcher_plugins::LauncherPluginsOutput;
use crate::components::launcher_plugins::actions::ActionsOutput;
use crate::components::launcher_plugins::applications::ApplicationsOutput;
use crate::components::launcher_plugins::simple::SimplePluginOutput;
use crate::components::launcher_plugins::websearch::WebSearchOutput;
use crate::components::nix_preview::{NixPreview, NixPreviewInit};
use crate::components::switch::SwitchOutput;
use crate::components::theme::{Style, StyleInit, StyleInput, StyleOutput};
use crate::components::windows::{Windows, WindowsInit, WindowsInput, WindowsOutput};
use crate::components::windows_overview::WindowsOverviewOutput;
use crate::structs;
use crate::util::default_config;
use relm4::ComponentController;
use relm4::abstractions::Toaster;
use relm4::adw::gtk::{Align, SelectionMode};
use relm4::adw::prelude::*;
use relm4::gtk::glib;
use relm4::{Component, ComponentParts, ComponentSender, Controller, SimpleComponent};
use relm4::{adw, gtk};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};
use std::path::Path;
use tracing::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct Root {
    config_file: Box<Path>,
    css_file: Box<Path>,

    is_generate_mode: bool,

    config: crate::Config,
    prev_config: crate::Config,

    footer: Controller<Footer>,
    alert_dialog: Controller<Alert>,
    launcher: Controller<Launcher>,
    windows: Controller<Windows>,
    changes: Controller<Changes>,
    nix_preview: Controller<NixPreview>,
    style: Controller<Style>,
    generate: Controller<Generate>,

    view_stack: adw::ViewStack,
    alert_dialog_changes_list: gtk::ListBox,
    toaster: Toaster,
}

#[derive(Debug)]
pub enum RootInput {
    Ignore,
    Reload(bool),
    CloseRequest,
    Close,
    Save(bool),
    Regenerate,
    AbortGenerate,

    Reset,
    SetConfig(crate::Config),
    SetPrevConfig(crate::Config),

    Windows(WindowsOutput),
    Launcher(LauncherOutput),
    Style(StyleOutput),
    Changes(ChangesOutput),
    Generate(GenerateOutput),
}

#[derive(Debug)]
pub struct RootInit {
    pub config_file: Box<Path>,
    pub system_data_dir: Box<Path>,
    pub css_file: Box<Path>,
    pub generate: bool,
}

#[derive(Debug)]
pub enum RootOutput {}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Root {
    type Init = RootInit;
    type Input = RootInput;
    type Output = RootOutput;

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some("Hyprshell Config Editor"),
            set_default_width: 900,
            set_default_height: 700,
            set_resizable: true,
            #[wrap(Some)]
            #[local_ref]
            set_content = toast_overlay -> adw::ToastOverlay {
                set_vexpand: true,
                adw::ToolbarView {
                    set_top_bar_style: adw::ToolbarStyle::Raised,
                    set_bottom_bar_style: adw::ToolbarStyle::Flat,
                    set_reveal_bottom_bars: true,
                    set_reveal_top_bars: true,
                    #[local_ref]
                    add_bottom_bar = footer -> gtk::ActionBar {},
                    #[wrap(Some)]
                    set_content = &adw::Clamp {
                        set_maximum_size: 1400,
                        gtk::ScrolledWindow {
                            #[local_ref]
                            view_stack -> adw::ViewStack {
                                set_enable_transitions: true,
                                set_hhomogeneous: false,
                                set_vhomogeneous: false,
                                set_transition_duration: 150,
                            }
                        }
                    },
                    add_top_bar = &adw::HeaderBar {
                        set_show_end_title_buttons: true,
                        set_show_start_title_buttons: true,
                        set_show_back_button: true,
                        #[wrap(Some)]
                        set_title_widget: view_stack_switcher = &adw::ViewSwitcherBar {
                            #[watch]
                            set_visible: !model.is_generate_mode,
                            set_reveal: true,
                        },
                        pack_start = &gtk::Button {
                            set_label: "Generate new",
                            set_css_classes: &["pill", "warning"],
                            #[watch]
                            set_visible: !model.is_generate_mode,
                            connect_clicked[sender] => move |_| {
                                sender.input(RootInput::Regenerate);
                            }
                        }
                    }
                },
            },
            connect_close_request[sender] => move |_| {
                sender.input(RootInput::CloseRequest);
                glib::Propagation::Stop
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = default_config();
        let config = structs::Config::from(config);

        let footer: Controller<Footer> = Footer::builder()
            .launch(FooterInit {
                config_file: init.config_file.clone(),
            })
            .forward(sender.input_sender(), |msg| match msg {
                FooterOutput::Reset => RootInput::Reset,
                FooterOutput::Close => RootInput::CloseRequest,
                FooterOutput::Save => RootInput::Save(false),
                FooterOutput::Reload => RootInput::Reload(false),
                FooterOutput::Abort => RootInput::AbortGenerate,
            });

        let changes_list = gtk::ListBox::builder()
            .css_classes(["items-list", "boxed-list"])
            .selection_mode(SelectionMode::None)
            .show_separators(false)
            .halign(Align::Center)
            .valign(Align::Start)
            .hexpand(true)
            .build();
        let alert_dialog = Alert::builder()
            .transient_for(&root)
            .launch(AlertSettings {
                text: Some("Do you want to close before saving?".to_string()),
                secondary_text: None,
                // secondary_text: Some(String::from("All unsaved changes will be lost")),
                confirm_label: Some(String::from("Close without saving")),
                cancel_label: Some(String::from("Cancel")),
                option_label: Some(String::from("Save and quit")),
                is_modal: true,
                destructive_accept: true,
                extra_child: Some(changes_list.clone().into()),
            })
            .forward(sender.input_sender(), |res| match res {
                AlertResponse::Confirm => RootInput::Close,
                AlertResponse::Option => RootInput::Save(true),
                AlertResponse::Cancel => RootInput::Ignore,
            });
        let style = Style::builder()
            .launch(StyleInit {
                system_data_dir: init.system_data_dir.clone(),
                css_file: init.css_file.clone(),
            })
            .forward(sender.input_sender(), RootInput::Style);
        let windows = Windows::builder()
            .launch(WindowsInit {
                config: config.windows.clone(),
            })
            .forward(sender.input_sender(), RootInput::Windows);
        let launcher = Launcher::builder()
            .launch(LauncherInit {
                config: config.windows.overview.launcher.clone(),
            })
            .forward(sender.input_sender(), RootInput::Launcher);
        let nix_preview = NixPreview::builder().launch(NixPreviewInit {}).detach();
        let changes = Changes::builder()
            .launch(ChangesInit {
                config: config.clone(),
            })
            .forward(sender.input_sender(), RootInput::Changes);
        let generate = Generate::builder()
            .launch(GenerateInit {
                system_data_dir: init.system_data_dir,
            })
            .forward(sender.input_sender(), RootInput::Generate);

        let view_stack = adw::ViewStack::builder().build();
        let toaster = Toaster::default();
        let model = Self {
            config_file: init.config_file,
            css_file: init.css_file.clone(),
            config: config.clone(),
            prev_config: config.clone(),
            is_generate_mode: init.generate,
            generate,
            footer,
            windows,
            launcher,
            changes,
            style,
            nix_preview,
            alert_dialog,
            toaster,
            alert_dialog_changes_list: changes_list,
            view_stack,
        };
        let toast_overlay = model.toaster.overlay_widget();
        let footer = model.footer.widget();
        let view_stack = &model.view_stack;
        let widgets = view_output!();

        model.view_stack.add_titled_with_icon(
            model.style.widget(),
            Some("style"),
            "Style",
            "viewimage",
        );
        model.view_stack.add_titled_with_icon(
            model.changes.widget(),
            None,
            "Changes",
            "document-edit-symbolic",
        );
        model.view_stack.add_titled_with_icon(
            model.nix_preview.widget(),
            None,
            "Nix Preview",
            "preview",
        );
        model.view_stack.add_titled_with_icon(
            model.windows.widget(),
            Some("overview"),
            "Windows",
            "configure",
        );
        if config.windows.overview.enabled {
            model.view_stack.add_titled_with_icon(
                model.launcher.widget(),
                Some("launcher"),
                "Launcher",
                "configure",
            );
        }
        model.view_stack.set_visible_child_name("overview");
        widgets
            .view_stack_switcher
            .set_stack(Some(&model.view_stack));
        sender.input(RootInput::Reload(true));
        ComponentParts { model, widgets }
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("root::update: {message:?}");
        match message {
            RootInput::Ignore => (),
            RootInput::CloseRequest => {
                debug!("close request");
                let changes = generate_items(
                    &self.alert_dialog_changes_list,
                    &self.config,
                    &self.prev_config,
                );
                if changes {
                    self.alert_dialog.emit(AlertMsg::Show);
                    self.alert_dialog.widgets().gtk_window_12.set_modal(true); // TODO remove if https://github.com/Relm4/Relm4/issues/837 fixed
                } else {
                    sender.input(RootInput::Close);
                }
            }
            RootInput::Close => {
                relm4::main_application().quit();
            }
            RootInput::Regenerate => {
                self.is_generate_mode = true;
                self.footer.emit(FooterInput::GenerateMode(true));
                self.generate.emit(GenerateInput::Start);
                trace!("Adding generate tab");
                if self.view_stack.child_by_name("generate").is_none() {
                    self.view_stack.add_titled_with_icon(
                        self.generate.widget(),
                        Some("generate"),
                        "Generate",
                        "configure",
                    );
                } else {
                    trace!("Generate tab already exists");
                }
                self.view_stack.set_visible_child_name("generate");
            }
            RootInput::AbortGenerate => {
                self.is_generate_mode = false;
                self.footer.emit(FooterInput::GenerateMode(false));
                if let Some(ch) = self.view_stack.child_by_name("generate") {
                    self.view_stack.remove(&ch);
                }
                self.view_stack.set_visible_child_name("overview");
            }
            RootInput::Generate(msg) => match msg {
                GenerateOutput::Finish(out) => {
                    self.is_generate_mode = false;
                    sender.input(RootInput::SetConfig(out));
                    self.footer.emit(FooterInput::GenerateMode(false));
                    if let Some(ch) = self.view_stack.child_by_name("generate") {
                        self.view_stack.remove(&ch);
                    }
                    self.view_stack.set_visible_child_name("overview");
                    sender.input(RootInput::Save(false));
                }
            },
            RootInput::Reload(initial) => {
                if self.config_file.exists() {
                    match config_lib::load_and_migrate_config(&self.config_file, true) {
                        Ok(c) => {
                            let config = structs::Config::from(c);
                            sender.input(RootInput::SetConfig(config.clone()));
                            sender.input(RootInput::SetPrevConfig(config));
                        }
                        Err(err) => {
                            warn!("Failed to load config: {err:#}");
                            self.toaster.add_toast(
                                adw::Toast::builder()
                                    .title(err.to_string())
                                    .timeout(0)
                                    .build(),
                            );
                        }
                    }
                } else if initial {
                    warn!("Config file doesnt exist");
                    sender.input(RootInput::Regenerate);
                } else {
                    warn!("Config file was deleted");
                    let button = adw::Toast::builder()
                        .title("Config file missing")
                        .button_label("Generate new")
                        .timeout(0)
                        .build();
                    let s = sender.clone();
                    button.connect_button_clicked(move |_| {
                        s.input(RootInput::Regenerate);
                    });
                    self.toaster.add_toast(button);

                    let config = default_config();
                    let config = structs::Config::from(config);
                    sender.input(RootInput::SetConfig(config.clone()));
                    sender.input(RootInput::SetPrevConfig(config));
                }
            }
            RootInput::SetConfig(config) => {
                self.config = config;

                if self.config.windows.overview.enabled {
                    trace!("Adding launcher tab");
                    if self.view_stack.child_by_name("launcher").is_none() {
                        self.view_stack.add_titled_with_icon(
                            self.launcher.widget(),
                            Some("launcher"),
                            "Launcher",
                            "configure",
                        );
                    } else {
                        trace!("Launcher tab already exists");
                    }
                } else if let Some(ch) = self.view_stack.child_by_name("launcher") {
                    self.view_stack.remove(&ch);
                }

                self.windows
                    .emit(WindowsInput::Set(self.config.windows.clone()));
                self.launcher.emit(LauncherInput::Set(
                    self.config.windows.overview.launcher.clone(),
                ));
                self.changes
                    .emit(ChangesInput::SetConfig(self.config.clone()));
            }
            RootInput::SetPrevConfig(config) => {
                self.prev_config = config;

                self.windows
                    .emit(WindowsInput::SetPrev(self.prev_config.windows.clone()));
                self.launcher.emit(LauncherInput::SetPrev(
                    self.prev_config.windows.overview.launcher.clone(),
                ));
                self.changes
                    .emit(ChangesInput::SetPrevConfig(self.prev_config.clone()));
            }
            RootInput::Save(close) => {
                match config_lib::write_config(
                    &self.config_file,
                    &(self.config.clone().into()),
                    true,
                ) {
                    Ok(()) => {
                        info!("Saved config to {}", self.config_file.display());
                        self.toaster.add_toast(
                            adw::Toast::builder()
                                .title("Saved".to_string())
                                .timeout(2)
                                .build(),
                        );
                    }
                    Err(err) => {
                        error!("Failed to save config: {err:#}");
                        self.toaster.add_toast(
                            adw::Toast::builder()
                                .title(err.to_string())
                                .timeout(0)
                                .build(),
                        );
                    }
                }

                if close {
                    sender.input(RootInput::Close);
                }
                sender.input(RootInput::SetPrevConfig(self.config.clone()));
            }
            RootInput::Reset => {
                self.config = self.prev_config.clone();

                self.windows.emit(WindowsInput::Reset);
                self.launcher.emit(LauncherInput::Reset);
                self.changes
                    .emit(ChangesInput::SetConfig(self.config.clone()));
            }
            RootInput::Changes(msg) => match msg {
                ChangesOutput::ChangesExist(changes_exist) => {
                    self.footer.emit(FooterInput::ChangesExist(changes_exist));
                }
            },
            RootInput::Style(msg) => match msg {
                StyleOutput::Apply((name, content)) => {
                    match std::fs::write(&self.css_file, content) {
                        Ok(()) => {
                            info!("Saved css from {name} to {}", self.css_file.display());
                            self.toaster.add_toast(
                                adw::Toast::builder()
                                    .title("Saved".to_string())
                                    .timeout(2)
                                    .build(),
                            );
                        }
                        Err(err) => {
                            error!("Failed to save css from {name}: {err:#}");
                            self.toaster.add_toast(
                                adw::Toast::builder()
                                    .title(err.to_string())
                                    .timeout(0)
                                    .build(),
                            );
                        }
                    }
                    self.style.emit(StyleInput::Reload);
                }
            },
            RootInput::Launcher(msg) => {
                let r#ref = &mut self.config.windows.overview.launcher;
                match msg {
                    LauncherOutput::Modifier(modifier) => {
                        r#ref.launch_modifier = modifier;
                    }
                    LauncherOutput::Width(width) => {
                        r#ref.width = width;
                    }
                    LauncherOutput::MaxItems(max_items) => {
                        r#ref.max_items = max_items;
                    }
                    LauncherOutput::DefaultTerminal(default_terminal) => match default_terminal {
                        None => {
                            r#ref.default_terminal = None;
                        }
                        Some(val) => {
                            r#ref.default_terminal = Some(val);
                        }
                    },
                    LauncherOutput::LauncherPlugins(msg) => match msg {
                        LauncherPluginsOutput::Applications(msg) => match msg {
                            ApplicationsOutput::Enabled(enabled) => {
                                r#ref.plugins.applications.enabled = enabled;
                            }
                            ApplicationsOutput::ShowExecs(enabled) => {
                                r#ref.plugins.applications.show_execs = enabled;
                            }
                            ApplicationsOutput::ShowActions(enabled) => {
                                r#ref.plugins.applications.show_actions_submenu = enabled;
                            }
                            ApplicationsOutput::CacheWeeks(weeks) => {
                                r#ref.plugins.applications.run_cache_weeks = weeks;
                            }
                        },
                        LauncherPluginsOutput::Terminal(msg) => match msg {
                            SimplePluginOutput::Enabled(enabled) => {
                                r#ref.plugins.terminal.enabled = enabled;
                            }
                        },
                        LauncherPluginsOutput::Shell(msg) => match msg {
                            SimplePluginOutput::Enabled(enabled) => {
                                r#ref.plugins.shell.enabled = enabled;
                            }
                        },
                        LauncherPluginsOutput::Calculator(msg) => match msg {
                            SimplePluginOutput::Enabled(enabled) => {
                                r#ref.plugins.calc.enabled = enabled;
                            }
                        },
                        LauncherPluginsOutput::FilePath(msg) => match msg {
                            SimplePluginOutput::Enabled(enabled) => {
                                r#ref.plugins.path.enabled = enabled;
                            }
                        },
                        LauncherPluginsOutput::WebSearch(msg) => match msg {
                            WebSearchOutput::Enabled(enabled) => {
                                r#ref.plugins.websearch.enabled = enabled;
                            }
                            WebSearchOutput::Engines(engines) => {
                                r#ref.plugins.websearch.engines = engines;
                            }
                        },
                        LauncherPluginsOutput::Actions(msg) => match msg {
                            ActionsOutput::Enabled(enabled) => {
                                r#ref.plugins.actions.enabled = enabled;
                            }
                            ActionsOutput::Actions(actions) => {
                                r#ref.plugins.actions.actions = actions;
                            }
                        },
                    },
                }
                // propagate event back
                sender.input(RootInput::SetConfig(self.config.clone()));
            }
            RootInput::Windows(msg) => {
                let r#ref = &mut self.config.windows;
                match msg {
                    WindowsOutput::Enabled(enabled) => {
                        r#ref.enabled = enabled;
                    }
                    WindowsOutput::Scale(scale) => {
                        r#ref.scale = scale;
                    }
                    WindowsOutput::ItemsPerRow(items_per_row) => {
                        r#ref.items_per_row = items_per_row;
                    }
                    WindowsOutput::Overview(msg) => match msg {
                        WindowsOverviewOutput::Enabled(enabled) => {
                            r#ref.overview.enabled = enabled;
                        }
                        WindowsOverviewOutput::Key(key) => r#ref.overview.key = key,
                        WindowsOverviewOutput::Modifier(modifier) => {
                            r#ref.overview.modifier = modifier;
                        }
                        WindowsOverviewOutput::FilterSameClass(enabled) => {
                            r#ref.overview.same_class = enabled;
                        }
                        WindowsOverviewOutput::FilterWorkspace(enabled) => {
                            r#ref.overview.current_workspace = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.overview.current_monitor = false;
                            }
                        }
                        WindowsOverviewOutput::FilterMonitor(enabled) => {
                            r#ref.overview.current_monitor = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.overview.current_workspace = false;
                            }
                        }
                        WindowsOverviewOutput::ExcludeSpecialWorkspaces(
                            exclude_special_workspaces,
                        ) => {
                            r#ref.overview.exclude_special_workspaces = exclude_special_workspaces;
                        }
                    },
                    WindowsOutput::Switch(msg) => match msg {
                        SwitchOutput::Enabled(enabled) => {
                            r#ref.switch.enabled = enabled;
                        }
                        SwitchOutput::Key(key) => r#ref.switch.key = key,
                        SwitchOutput::Modifier(modifier) => {
                            r#ref.switch.modifier = modifier;
                        }
                        SwitchOutput::FilterSameClass(enabled) => {
                            r#ref.switch.same_class = enabled;
                        }
                        SwitchOutput::FilterWorkspace(enabled) => {
                            r#ref.switch.current_workspace = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.switch.current_monitor = false;
                            }
                        }
                        SwitchOutput::FilterMonitor(enabled) => {
                            r#ref.switch.current_monitor = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.switch.current_workspace = false;
                            }
                        }
                        SwitchOutput::SwitchWorkspaces(enabled) => {
                            r#ref.switch.switch_workspaces = enabled;
                        }
                        SwitchOutput::ExcludeSpecialWorkspaces(exclude_special_workspaces) => {
                            r#ref.switch.exclude_special_workspaces = exclude_special_workspaces;
                        }
                    },
                    WindowsOutput::Switch2(msg) => match msg {
                        SwitchOutput::Enabled(enabled) => {
                            r#ref.switch_2.enabled = enabled;
                        }
                        SwitchOutput::Key(key) => r#ref.switch_2.key = key,
                        SwitchOutput::Modifier(modifier) => {
                            r#ref.switch_2.modifier = modifier;
                        }
                        SwitchOutput::FilterSameClass(enabled) => {
                            r#ref.switch_2.same_class = enabled;
                        }
                        SwitchOutput::FilterWorkspace(enabled) => {
                            r#ref.switch_2.current_workspace = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.switch_2.current_monitor = false;
                            }
                        }
                        SwitchOutput::FilterMonitor(enabled) => {
                            r#ref.switch_2.current_monitor = enabled;
                            // current monitor and current workspace are incompatible
                            if enabled {
                                r#ref.switch_2.current_workspace = false;
                            }
                        }
                        SwitchOutput::SwitchWorkspaces(enabled) => {
                            r#ref.switch_2.switch_workspaces = enabled;
                        }
                        SwitchOutput::ExcludeSpecialWorkspaces(exclude_special_workspaces) => {
                            r#ref.switch_2.exclude_special_workspaces = exclude_special_workspaces;
                        }
                    },
                }
                // propagate event back
                sender.input(RootInput::SetConfig(self.config.clone()));
            }
        }
    }
}
