use crate::components::footer::*;
use crate::components::nix_preview::*;
use crate::components::switch::SwitchOutput;
use crate::components::windows::*;
use config_lib::io::{self, RawIOConfig};
use core_lib::binds::get_hyprshell_path;
use relm4::actions::AccelsPlus;
use relm4::actions::ActionName;
use relm4::actions::RelmAction;
use relm4::actions::RelmActionGroup;
use relm4::new_action_group;
use relm4::new_stateless_action;
use relm4::prelude::*;
use relm4::{abstractions::Toaster, adw::prelude::*};
use relm4_components::alert::{Alert, AlertResponse, AlertSettings};

use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use tracing::{debug, error, info, trace, warn};

const HELP_URL: &str = concat!(
    "https://github.com/H3rmt/hyprshell/tree/v",
    env!("CARGO_PKG_VERSION"),
);

#[derive(Debug)]
pub struct Root {
    config_file: Rc<Path>,
    css_file: Rc<Path>,

    is_generate_mode: bool,
    is_show_sidebar: bool,

    config: io::ConfigFile,
    prev_config: config_lib::io::Config,

    footer: Controller<Footer>,
    alert_dialog: Controller<Alert>,
    windows: Controller<Windows>,
    // launcher: Controller<Launcher>,
    // changes: Controller<Changes>,
    nix_preview: Controller<NixPreview>,
    // style: Controller<Style>,
    // generate: Controller<Generate>,
    view_stack: adw::ViewStack,
    alert_dialog_changes_list: gtk::ListBox,
    toaster: Toaster,
    menu: FactoryVecDeque<MenuItem>,
}

#[derive(Debug)]
pub enum RootInput {
    Ignore,
    /// bool: intial config (changes no config behaviour)
    Reload(bool),
    CloseRequest,
    Close,
    /// bool: close after this save
    Save(bool),
    Regenerate,
    AbortGenerate,
    ReloadHyprshell,
    OpenAbout,
    ClosePopover,

    Reset,
    SetConfig(io::ConfigFile),
    SetPrevConfig(io::Config),
    DispatchConfigUpdate,
    DispatchPrevConfigUpdate,

    Windows(WindowsOutput),
    // Launcher(LauncherOutput),
    // Style(StyleOutput),
    // Changes(ChangesOutput),
    // Generate(GenerateOutput),

    // TODO dont use this
    Toast(adw::Toast),
}

#[derive(Debug)]
pub enum RootOutput {}

#[derive(Debug)]
pub struct RootInit {
    pub config_file: Rc<Path>,
    pub css_path: Rc<Path>,
    pub system_data_dirs: Rc<[Box<Path>]>,
    pub generate: bool,
}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Root {
    type Init = RootInit;
    type Input = RootInput;
    type Output = RootOutput;

    #[allow(clippy::too_many_lines)]
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let group = setup_keybinds_and_actions(&sender);
        group.register_for_widget(&root);

        let mut menu: FactoryVecDeque<MenuItem> = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();
        // .forward(sender.input_sender(), |msg| match msg {
        //     MenuItemOutput::Close => RootInput::ClosePopover,
        // });

        let mut l = menu.guard();
        for conf in vec![
            MenuItemInit {
                action: Some(ReloadHyprshell::action_name()),
                red: true,
                icon_name: Some("view-refresh-symbolic"),
                text: "Force Reload Daemon",
            },
            MenuItemInit {
                action: Some(ReloadConfig::action_name()),
                icon_name: Some("view-refresh-symbolic"),
                red: true,
                text: "Force Reload from Disk",
            },
            MenuItemInit {
                action: Some(OpenHelp::action_name()),
                icon_name: Some("info-outline-symbolic"),
                red: false,
                text: "Open Help (GitHub)",
            },
            MenuItemInit {
                action: Some(OpenAbout::action_name()),
                icon_name: Some("help-about"),
                red: false,
                text: "About Hyprshell Settings Editor",
            },
        ] {
            l.push_back(conf);
        }
        drop(l);

        let footer: Controller<Footer> = Footer::builder()
            .launch(FooterInit {
                config_file: init.config_file.clone(),
            })
            .forward(sender.input_sender(), |msg| match msg {
                FooterOutput::Reset => RootInput::Reset,
                FooterOutput::Close => RootInput::CloseRequest,
                FooterOutput::Save => RootInput::Save(false),
                // FooterOutput::ReloadConfig => RootInput::Reload(false),
                // FooterOutput::Abort => RootInput::AbortGenerate,
                // FooterOutput::ReloadHyprshell => RootInput::ReloadHyprshell,
            });

        let changes_list = gtk::ListBox::builder()
            .css_classes(["items-list", "boxed-list"])
            .selection_mode(gtk::SelectionMode::None)
            .show_separators(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
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
        // let style = Style::builder()
        //     .launch(StyleInit {
        //         system_data_dirs: init.system_data_dirs.clone(),
        //         css_path: init.css_path.clone(),
        //     })
        //     .forward(sender.input_sender(), RootInput::Style);
        let windows = Windows::builder()
            .launch(WindowsInit {})
            .forward(sender.input_sender(), RootInput::Windows);
        // let launcher = Launcher::builder()
        //     .launch(LauncherInit {
        //         config: config.clone(),
        //     })
        //     .forward(sender.input_sender(), RootInput::Launcher);
        let nix_preview = NixPreview::builder().launch(NixPreviewInit {}).detach();
        // let changes = Changes::builder()
        //     .launch(ChangesInit {
        //         config: config.clone(),
        //     })
        //     .forward(sender.input_sender(), RootInput::Changes);
        // let generate = Generate::builder()
        //     .launch(GenerateInit {
        //         system_data_dirs: init.system_data_dirs,
        //     })
        //     .forward(sender.input_sender(), RootInput::Generate);

        let config = io::Config::default();
        let view_stack = adw::ViewStack::builder().build();
        let toaster = Toaster::default();
        let model = Self {
            config_file: init.config_file,
            css_file: init.css_path.clone(),
            config: io::ConfigFile::new_without_file(config.clone()),
            prev_config: config,
            is_generate_mode: init.generate,
            is_show_sidebar: true,
            // generate,
            footer,
            windows,
            // launcher,
            // changes,
            // style,
            nix_preview,
            alert_dialog,
            toaster,
            alert_dialog_changes_list: changes_list,
            view_stack,
            menu,
        };
        let toast_overlay = model.toaster.overlay_widget();
        let footer = model.footer.widget();
        let menu = model.menu.widget();
        let popover = gtk::Popover::new();
        let view_stack = &model.view_stack;
        let widgets = view_output!();

        // model.view_stack.add_titled_with_icon(
        //     model.style.widget(),
        //     Some("style"),
        //     "Style",
        //     "viewimage",
        // );
        // model.view_stack.add_titled_with_icon(
        //     model.changes.widget(),
        //     None,
        //     "Changes",
        //     "document-edit-symbolic",
        // );
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
        // if config.get_raw_config().windows.is_some() {
        //     model.view_stack.add_titled_with_icon(
        //         model.launcher.widget(),
        //         Some("launcher"),
        //         "Launcher",
        //         "configure",
        //     );
        // }
        model.view_stack.set_visible_child_name("overview");
        widgets
            .view_stack_switcher
            .set_stack(Some(&model.view_stack));
        sender.input(RootInput::Reload(true));
        ComponentParts { model, widgets }
    }

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
                    // set_reveal_bottom_bars: true,
                    // set_reveal_top_bars: true,
                    add_top_bar = &adw::HeaderBar {
                      //           set_show_end_title_buttons: true,
                      //           set_show_start_title_buttons: true,
                      //           set_show_back_button: true,
                      //           // pack_start = &gtk::Button {
                      //           //     set_label: "Generate new",
                      //           //     set_css_classes: &["pill", "warning"],
                      //           //     #[watch]
                      //           //     set_visible: !model.is_generate_mode,
                      //           //     connect_clicked[sender] => move |_| {
                      //           //         sender.input(RootInput::Regenerate);
                      //           //     }
                      //           // }
                    },
                    adw::OverlaySplitView {
                      #[watch]
                      set_collapsed: !model.is_show_sidebar,
                      #[wrap(Some)]
                      set_content = &adw::ToolbarView {
                        set_bottom_bar_style: adw::ToolbarStyle::Flat,
                        #[local_ref]
                        add_bottom_bar = footer -> gtk::ActionBar {},
                        adw::Clamp {
                          set_maximum_size: 1000,
                          gtk::ScrolledWindow {
                            #[local_ref]
                            view_stack -> adw::ViewStack {
                                // TODO check
                                set_enable_transitions: true,
                                set_hhomogeneous: false,
                                set_vhomogeneous: false,
                                set_transition_duration: 110,
                            }
                          }
                        }
                      },
                      #[wrap(Some)]
                      set_sidebar: view_stack_switcher = &adw::ViewSwitcherSidebar {
                          set_expand: true,
                          set_mode: adw::SidebarMode::Sidebar,
                          #[wrap(Some)]
                          set_suffix = &gtk::Box {
                            set_valign: gtk::Align::End,
                            set_halign: gtk::Align::Fill,
                            set_vexpand: true,
                            set_spacing: 1,
                            set_orientation: gtk::Orientation::Horizontal,
                            set_margin_all: 4,
                            // gtk::Box {
                              // gtk::Button {
                              //     set_label: "Force Reload Daemon",
                              //     set_css_classes: &["destructive-action"],
                              //     // connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::ReloadHyprshell),
                              // },
                              // gtk::Button {
                              //     set_label: "Force Reload from Disk",
                              //     set_css_classes: &["destructive-action"],
                              //     // connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Reload),
                              // },
                            // },
                            gtk::Label {
                              // set_css_classes: &["test-colored"],
                              set_hexpand: true,
                              set_halign: gtk::Align::Start,
                              set_valign: gtk::Align::End,
                              set_margin_bottom: 5,
                              set_margin_start: 5,
                              set_markup: &format!("<a href=\"{}\">Hyprshell v{}</a>", HELP_URL, env!("CARGO_PKG_VERSION")),
                            },
                            gtk::MenuButton {
                              set_hexpand: true,
                              set_halign: gtk::Align::End,
                              set_icon_name: "open-menu-symbolic",
                              #[wrap(Some)]
                              #[local_ref]
                              set_popover = &popover -> gtk::Popover {
                                #[local_ref]
                                menu -> gtk::Box {
                                  set_orientation: gtk::Orientation::Vertical,
                                  set_spacing: 0,
                                  set_valign: gtk::Align::Center,
                                }
                              },
                            }
                          },
                      },
                    },
                },
            },
            connect_close_request[sender] => move |_| {
                sender.input(RootInput::CloseRequest);
                gtk::glib::Propagation::Stop
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("root::update: {message:?}");
        match message {
            RootInput::Ignore => (),
            RootInput::ClosePopover => (),
            RootInput::OpenAbout => {
                // hacky way to get root window
                let u = self.alert_dialog.widget();
                adw::AboutDialog::builder()
                  .application_icon("hyprshell-settings")
                  .application_name("Hyprshell Settings Editor")
                  .developer_name("H3rmt")
                  .issue_url("https://github.com/H3rmt/hyprshell/issues")
                  .license(include_str!("../../../../LICENSE"))
                  .release_notes("<p>RELEASE NOTES</p>\n<ul><li>a</li><li>b</li></ul><p><code>let test = 23;</code></p>")
                  // .release_notes_version(env!("CARGO_PKG_VERSION"))
                  .version(env!("CARGO_PKG_VERSION"))
                  .website(HELP_URL)
                  .build()
                  .present(Some(u));
            }
            RootInput::CloseRequest => {
                debug!("close request");
                // let changes = generate_items(
                //     &self.alert_dialog_changes_list,
                //     &self.config,
                //     &self.prev_config,
                // );
                // if changes {
                //     self.alert_dialog.emit(AlertMsg::Show);
                // } else {
                sender.input(RootInput::Close);
                // }
            }
            RootInput::Close => {
                relm4::main_application().quit();
            }
            RootInput::Regenerate => {
                // self.is_generate_mode = true;
                // self.footer.emit(FooterInput::GenerateMode(true));
                // self.generate.emit(GenerateInput::Start);
                // trace!("Adding generate tab");
                // if self.view_stack.child_by_name("generate").is_none() {
                //     self.view_stack.add_titled_with_icon(
                //         self.generate.widget(),
                //         Some("generate"),
                //         "Generate",
                //         "configure",
                //     );
                // } else {
                //     trace!("Generate tab already exists");
                // }
                // self.view_stack.set_visible_child_name("generate");
            }
            RootInput::AbortGenerate => {
                // self.is_generate_mode = false;
                // self.footer.emit(FooterInput::GenerateMode(false));
                // if let Some(ch) = self.view_stack.child_by_name("generate") {
                //     self.view_stack.remove(&ch);
                // }
                // self.view_stack.set_visible_child_name("overview");
            }
            // RootInput::Generate(msg) => match msg {
            //     GenerateOutput::Finish(out) => {
            //         self.is_generate_mode = false;
            //         sender.input(RootInput::SetConfig(out));
            //         self.footer.emit(FooterInput::GenerateMode(false));
            //         if let Some(ch) = self.view_stack.child_by_name("generate") {
            //             self.view_stack.remove(&ch);
            //         }
            //         self.view_stack.set_visible_child_name("overview");
            //         sender.input(RootInput::Save(false));
            //     }
            // },
            RootInput::ReloadHyprshell => {
                let mut cmd = Command::new(get_hyprshell_path());
                cmd.arg("socat").arg(core_lib::binds::generate_transfer(
                    &core_lib::transfer::ExternalTransferType::Reload,
                ));
                if let Ok(output) = cmd.output().inspect_err(|err| {
                    sender.input_sender().emit(RootInput::Toast(
                        adw::Toast::builder()
                            .title(format!(r"Failed to reload hyprshell:{err:?}"))
                            .timeout(0)
                            .build(),
                    ));
                    error!("Failed to reload hyprshell: {err:#}");
                }) && !output.status.success()
                {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.is_empty() {
                        sender.input_sender().emit(RootInput::Toast(
                            adw::Toast::builder()
                                .title(format!(
                                    "Failed to reload hyprshell (exit code: {})",
                                    output.status
                                ))
                                .timeout(0)
                                .use_markup(true)
                                .build(),
                        ));
                    } else {
                        sender.input_sender().emit(RootInput::Toast(
                            adw::Toast::builder()
                                .title(r"Failed to reload hyprshell".to_string())
                                .timeout(2)
                                .build(),
                        ));
                        sender.input_sender().emit(RootInput::Toast(
                            adw::Toast::builder()
                                .title(format!(r"<small><tt>{stderr}</tt></small>"))
                                .timeout(0)
                                .use_markup(true)
                                .build(),
                        ));
                    }
                    error!("Failed to reload hyprshell: {output:?}");
                }
            }
            RootInput::Reload(initial) => {
                if self.config_file.exists() {
                    match config_lib::load_and_migrate_io_config(&self.config_file, true) {
                        Ok(c) => {
                            sender.input(RootInput::SetPrevConfig(c.get_raw_config().clone()));
                            sender.input(RootInput::SetConfig(c));
                        }
                        Err(err) => {
                            warn!("Failed to load config: {err:#}");
                            sender.input(RootInput::Toast(
                                adw::Toast::builder()
                                    .title(err.to_string())
                                    .timeout(0)
                                    .build(),
                            ));
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
                    sender.input(RootInput::Toast(button));

                    let config = io::Config::default();
                    sender.input(RootInput::SetConfig(io::ConfigFile::new_without_file(
                        config.clone(),
                    )));
                    sender.input(RootInput::SetPrevConfig(config));
                }
            }
            RootInput::SetConfig(config) => {
                self.config = config;
                sender.input(RootInput::DispatchConfigUpdate);
            }
            RootInput::SetPrevConfig(config) => {
                self.prev_config = config;
                sender.input(RootInput::DispatchPrevConfigUpdate);
            }
            RootInput::DispatchConfigUpdate => {
                // if self.config.windows.overview.enabled {
                //     trace!("Adding launcher tab");
                //     if self.view_stack.child_by_name("launcher").is_none() {
                //         self.view_stack.add_titled_with_icon(
                //             self.launcher.widget(),
                //             Some("launcher"),
                //             "Launcher",
                //             "configure",
                //         );
                //     } else {
                //         trace!("Launcher tab already exists");
                //     }
                // } else if let Some(ch) = self.view_stack.child_by_name("launcher") {
                //     self.view_stack.remove(&ch);
                // }

                let io_config = self.config.get_raw_config();
                self.windows
                    .emit(WindowsInput::Set(io_config.windows.clone()));
                // self.launcher.emit(LauncherInput::Set(
                //     self.config.windows.overview.launcher.clone(),
                // ));
                // self.changes
                //     .emit(ChangesInput::SetConfig(self.config.clone()));
            }
            RootInput::DispatchPrevConfigUpdate => {
                // let prev_io_config = self.prev_config;

                // self.windows
                //     .emit(WindowsInput::SetPrev(prev_io_config.windows.clone()));
                // self.launcher.emit(LauncherInput::SetPrev(
                //     self.prev_config.windows.overview.launcher.clone(),
                // ));
                // self.changes
                //     .emit(ChangesInput::SetPrevConfig(self.prev_config.clone()));
            }

            RootInput::Save(close) => {
                // let config: &mut ConfigFile = self.config.clone().into();
                let config: &mut io::ConfigFile = todo!("todo");
                match config_lib::write_io_config(&self.config_file, config, true) {
                    Ok(()) => {
                        info!("Saved config to {}", self.config_file.display());
                        sender.input(RootInput::Toast(
                            adw::Toast::builder()
                                .title("Saved".to_string())
                                .timeout(2)
                                .build(),
                        ));
                    }
                    Err(err) => {
                        error!("Failed to save config: {err:#}");
                        sender.input(RootInput::Toast(
                            adw::Toast::builder()
                                .title(err.to_string())
                                .timeout(0)
                                .build(),
                        ));
                    }
                }

                if close {
                    sender.input(RootInput::Close);
                }
                sender.input(RootInput::SetPrevConfig(
                    self.config.get_raw_config().clone(),
                ));
            }
            RootInput::Reset => {
                *(self.config.get_raw_config_mut()) = self.prev_config.clone();

                // self.windows.emit(WindowsInput::Reset);
                // self.launcher.emit(LauncherInput::Reset);
                // self.changes
                //     .emit(ChangesInput::SetConfig(self.config.clone()));
            }
            // RootInput::Changes(msg) => match msg {
            //     ChangesOutput::ChangesExist(changes_exist) => {
            //         self.footer.emit(FooterInput::ChangesExist(changes_exist));
            //     }
            // },
            RootInput::Toast(toast) => {
                self.toaster.add_toast(toast);
            } // RootInput::Style(msg) => match msg {
            //     StyleOutput::Apply((name, content)) => {
            //         match std::fs::write(&self.css_file, content) {
            //             Ok(()) => {
            //                 info!("Saved css from {name} to {}", self.css_file.display());
            //                 sender.input(RootInput::Toast(
            //                     adw::Toast::builder()
            //                         .title("Saved".to_string())
            //                         .timeout(2)
            //                         .build(),
            //                 ));
            //             }
            //             Err(err) => {
            //                 error!("Failed to save css from {name}: {err:#}");
            //                 sender.input(RootInput::Toast(
            //                     adw::Toast::builder()
            //                         .title(err.to_string())
            //                         .timeout(0)
            //                         .build(),
            //                 ));
            //             }
            //         }
            //         self.style.emit(StyleInput::Reload);
            //     }
            //     StyleOutput::Toast((msg, timeout)) => sender.input(RootInput::Toast(
            //         adw::Toast::builder().title(msg).timeout(timeout).build(),
            //     )),
            // },
            // RootInput::Launcher(msg) => {
            //     let r#ref = &mut self.config.windows.overview.launcher;
            //     match msg {
            //         LauncherOutput::Modifier(modifier) => {
            //             r#ref.launch_modifier = modifier;
            //         }
            //         LauncherOutput::Width(width) => {
            //             r#ref.width = width;
            //         }
            //         LauncherOutput::MaxItems(max_items) => {
            //             r#ref.max_items = max_items;
            //         }
            //         LauncherOutput::DefaultTerminal(default_terminal) => match default_terminal {
            //             None => {
            //                 r#ref.default_terminal = None;
            //             }
            //             Some(val) => {
            //                 r#ref.default_terminal = Some(val);
            //             }
            //         },
            //         LauncherOutput::LauncherPlugins(msg) => match msg {
            //             LauncherPluginsOutput::Applications(msg) => match msg {
            //                 ApplicationsOutput::Enabled(enabled) => {
            //                     r#ref.plugins.applications.enabled = enabled;
            //                 }
            //                 ApplicationsOutput::ShowExecs(enabled) => {
            //                     r#ref.plugins.applications.show_execs = enabled;
            //                 }
            //                 ApplicationsOutput::ShowActions(enabled) => {
            //                     r#ref.plugins.applications.show_actions_submenu = enabled;
            //                 }
            //                 ApplicationsOutput::CacheWeeks(weeks) => {
            //                     r#ref.plugins.applications.run_cache_weeks = weeks;
            //                 }
            //             },
            //             LauncherPluginsOutput::Terminal(msg) => match msg {
            //                 SimplePluginOutput::Enabled(enabled) => {
            //                     r#ref.plugins.terminal.enabled = enabled;
            //                 }
            //             },
            //             LauncherPluginsOutput::Shell(msg) => match msg {
            //                 SimplePluginOutput::Enabled(enabled) => {
            //                     r#ref.plugins.shell.enabled = enabled;
            //                 }
            //             },
            //             LauncherPluginsOutput::Calculator(msg) => match msg {
            //                 CalcPluginOutput::Enabled(enabled) => {
            //                     r#ref.plugins.calc.enabled = enabled;
            //                 }
            //                 CalcPluginOutput::SetPrefix(prefix) => {
            //                     r#ref.plugins.calc.prefix = prefix;
            //                 }
            //             },
            //             LauncherPluginsOutput::FilePath(msg) => match msg {
            //                 SimplePluginOutput::Enabled(enabled) => {
            //                     r#ref.plugins.path.enabled = enabled;
            //                 }
            //             },
            //             LauncherPluginsOutput::WebSearch(msg) => match msg {
            //                 WebSearchOutput::Enabled(enabled) => {
            //                     r#ref.plugins.websearch.enabled = enabled;
            //                 }
            //                 WebSearchOutput::Engines(engines) => {
            //                     r#ref.plugins.websearch.engines = engines;
            //                 }
            //             },
            //             LauncherPluginsOutput::Actions(msg) => match msg {
            //                 ActionsOutput::Enabled(enabled) => {
            //                     r#ref.plugins.actions.enabled = enabled;
            //                 }
            //                 ActionsOutput::Actions(actions) => {
            //                     r#ref.plugins.actions.actions = actions;
            //                 }
            //             },
            //         },
            //     }
            //     // propagate event back
            //     sender.input(RootInput::SetConfig(self.config.clone()));
            // }
            RootInput::Windows(msg) => {
                let cfg = self.config.get_raw_config_mut();
                match msg {
                    WindowsOutput::Enabled(config) => {
                        cfg.windows = config;
                    }
                    WindowsOutput::Scale(scale) => {
                        if let Some(windows) = cfg.windows.as_mut() {
                            windows.scale = scale;
                        }
                    }
                    WindowsOutput::ItemsPerRow(items_per_row) => {
                        if let Some(windows) = cfg.windows.as_mut() {
                            windows.items_per_row = items_per_row;
                        }
                    } //     WindowsOutput::Overview(msg) => match msg {
                    //         WindowsOverviewOutput::Enabled(enabled) => {
                    //             r#ref.overview.enabled = enabled;
                    //         }
                    //         WindowsOverviewOutput::Key(key) => r#ref.overview.key = key,
                    //         WindowsOverviewOutput::Modifier(modifier) => {
                    //             r#ref.overview.modifier = modifier;
                    //         }
                    //         WindowsOverviewOutput::FilterSameClass(enabled) => {
                    //             r#ref.overview.same_class = enabled;
                    //         }
                    //         WindowsOverviewOutput::FilterWorkspace(enabled) => {
                    //             r#ref.overview.current_workspace = enabled;
                    //             // current monitor and current workspace are incompatible
                    //             if enabled {
                    //                 r#ref.overview.current_monitor = false;
                    //             }
                    //         }
                    //         WindowsOverviewOutput::FilterMonitor(enabled) => {
                    //             r#ref.overview.current_monitor = enabled;
                    //             // current monitor and current workspace are incompatible
                    //             if enabled {
                    //                 r#ref.overview.current_workspace = false;
                    //             }
                    //         }
                    //         WindowsOverviewOutput::ExcludeWorkspaces(exclude_workspaces) => {
                    //             r#ref.overview.exclude_workspaces = exclude_workspaces;
                    //         }
                    //         WindowsOverviewOutput::TopOffset(top_offset) => {
                    //             r#ref.overview.top_offset = top_offset;
                    //         }
                    //     },
                    WindowsOutput::Switch(msg) => match msg {
                        SwitchOutput::Enabled(config) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                windows.switch = config;
                            }
                        }
                        SwitchOutput::Key(key) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    switch.key = key.into_boxed_str();
                                };
                            }
                        }
                        SwitchOutput::Modifier(modifier) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    switch.modifier = modifier;
                                };
                            }
                        }
                        SwitchOutput::FilterSameClass(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    if enabled {
                                        switch.filter_by.push(config_lib::io::FilterBy::SameClass);
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::SameClass
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::FilterWorkspace(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    if enabled {
                                        switch
                                            .filter_by
                                            .push(config_lib::io::FilterBy::CurrentWorkspace);
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentMonitor
                                        });
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentWorkspace
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::FilterMonitor(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    if enabled {
                                        switch
                                            .filter_by
                                            .push(config_lib::io::FilterBy::CurrentMonitor);
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentWorkspace
                                        });
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentMonitor
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::SwitchWorkspaces(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    switch.switch_workspaces = enabled;
                                };
                            }
                        }
                        SwitchOutput::ExcludeWorkspaces(exclude_workspaces) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    switch.exclude_workspaces = exclude_workspaces.into_boxed_str();
                                };
                            }
                        }
                        SwitchOutput::KillKey(key) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch.as_mut() {
                                    switch.kill_key = key.unwrap_or('\0');
                                };
                            }
                        }
                    },
                    WindowsOutput::Switch2(msg) => match msg {
                        SwitchOutput::Enabled(config) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                windows.switch_2 = config;
                            }
                        }
                        SwitchOutput::Key(key) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    switch.key = key.into_boxed_str();
                                };
                            }
                        }
                        SwitchOutput::Modifier(modifier) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    switch.modifier = modifier;
                                };
                            }
                        }
                        SwitchOutput::FilterSameClass(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    if enabled {
                                        switch.filter_by.push(config_lib::io::FilterBy::SameClass);
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::SameClass
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::FilterWorkspace(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    if enabled {
                                        switch
                                            .filter_by
                                            .push(config_lib::io::FilterBy::CurrentWorkspace);
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentMonitor
                                        });
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentWorkspace
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::FilterMonitor(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    if enabled {
                                        switch
                                            .filter_by
                                            .push(config_lib::io::FilterBy::CurrentMonitor);
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentWorkspace
                                        });
                                    } else {
                                        switch.filter_by.retain(|filter| {
                                            *filter != config_lib::io::FilterBy::CurrentMonitor
                                        });
                                    }
                                };
                            }
                        }
                        SwitchOutput::SwitchWorkspaces(enabled) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    switch.switch_workspaces = enabled;
                                };
                            }
                        }
                        SwitchOutput::ExcludeWorkspaces(exclude_workspaces) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    switch.exclude_workspaces = exclude_workspaces.into_boxed_str();
                                };
                            }
                        }
                        SwitchOutput::KillKey(key) => {
                            if let Some(windows) = cfg.windows.as_mut() {
                                if let Some(switch) = windows.switch_2.as_mut() {
                                    switch.kill_key = key.unwrap_or('\0');
                                };
                            }
                        }
                    },
                }
                // propagate event back
                sender.input(RootInput::DispatchConfigUpdate);
            }
        }
    }
}

fn setup_keybinds_and_actions(
    sender: &ComponentSender<Root>,
) -> RelmActionGroup<WindowActionGroup> {
    let mut group = RelmActionGroup::<WindowActionGroup>::new();
    let app = relm4::main_application();

    app.set_accelerators_for_action::<OpenHelp>(&["F1"]);
    group.add_action(RelmAction::<OpenHelp>::new_stateless(move |_| {
        tracing::debug!("opened help: {HELP_URL}");
        gtk::UriLauncher::new(HELP_URL).launch(
            None::<&gtk::Window>,
            None::<&gtk::gio::Cancellable>,
            move |result| {
                if let Err(e) = result {
                    tracing::warn!("Failed to open URL({HELP_URL}): {e}");
                }
            },
        );
    }));

    app.set_accelerators_for_action::<SaveConfig>(&["<Control>S"]);
    let send = sender.clone();
    group.add_action(RelmAction::<SaveConfig>::new_stateless(move |_| {
        send.input_sender().emit(RootInput::Save(true));
    }));

    app.set_accelerators_for_action::<ReloadConfig>(&["<Control>R"]);
    let send = sender.clone();
    group.add_action(RelmAction::<ReloadConfig>::new_stateless(move |_| {
        send.input_sender().emit(RootInput::Reload(false));
    }));

    app.set_accelerators_for_action::<ReloadHyprshell>(&["<Control><SHIFT>R"]);
    let send = sender.clone();
    group.add_action(RelmAction::<ReloadHyprshell>::new_stateless(move |_| {
        send.input_sender().emit(RootInput::ReloadHyprshell);
    }));

    let send = sender.clone();
    group.add_action(RelmAction::<OpenAbout>::new_stateless(move |_| {
        send.input_sender().emit(RootInput::OpenAbout);
    }));
    group
}

new_action_group!(WindowActionGroup, "win");
new_stateless_action!(ReloadHyprshell, WindowActionGroup, "reload-hyprshell");
new_stateless_action!(ReloadConfig, WindowActionGroup, "reload-config");
new_stateless_action!(SaveConfig, WindowActionGroup, "save-config");
new_stateless_action!(OpenHelp, WindowActionGroup, "open-help");
new_stateless_action!(OpenAbout, WindowActionGroup, "open-about");

#[derive(Debug)]
pub struct MenuItem {
    action: Option<String>,
    text: &'static str,
    accel: String,
    start_icon_name: Option<&'static str>,
    red: bool,
}

impl MenuItem {
    fn accelerator_label(action: Option<&str>) -> String {
        action
            .and_then(|action| {
                relm4::main_application()
                    .accels_for_action(action)
                    .first()
                    .and_then(|accel| gtk::accelerator_parse(accel.as_str()))
                    .map(|(key, modifiers)| gtk::accelerator_get_label(key, modifiers).to_string())
            })
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum MenuItemInput {}

#[derive(Debug)]
pub struct MenuItemInit {
    action: Option<String>,
    text: &'static str,
    icon_name: Option<&'static str>,
    red: bool,
}

#[derive(Debug)]
pub enum MenuItemOutput {
    Close,
}

#[relm4::factory(pub)]
impl FactoryComponent for MenuItem {
    type Init = MenuItemInit;
    type Input = MenuItemInput;
    type Output = MenuItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        let accel = Self::accelerator_label(init.action.as_deref());
        Self {
            action: init.action,
            text: init.text,
            accel,
            start_icon_name: init.icon_name,
            red: init.red,
        }
    }

    view! {
      #[root]
      gtk::Button {
        set_action_name: self.action.as_deref(),
        set_hexpand: true,
        set_halign: gtk::Align::Fill,
        set_css_classes: &["flat", "popover-action-row"],
        connect_clicked[sender] => move |_| sender.output_sender().emit(MenuItemOutput::Close),
        gtk::Box {
          set_orientation: gtk::Orientation::Horizontal,
          set_spacing: 12,
          set_hexpand: true,
          match &self.start_icon_name {
            Some(start_icon_name) => {
              gtk::Image {
                #[watch]
                set_icon_name: Some(start_icon_name)
              }
            }
            None => {
              gtk::Image {
                set_icon_name: None
              }
            },
          },
          gtk::Label::new(Some(&self.text)) {
            set_halign: gtk::Align::Start,
            set_hexpand: true,
            set_css_classes: if self.red { &["red-label", "popover-action-label"] } else { &["popover-action-label"] },
          },
          gtk::Label::new(Some(&self.accel)) {
            set_halign: gtk::Align::End,
            set_hexpand: true,
            set_css_classes: &["dim-label"],
          }
        }
      }
    }
}
