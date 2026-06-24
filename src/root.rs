use crate::util;
use crate::wm::configure_wm;
use anyhow::Context;
use async_channel::Receiver;
use core_lib::listener::hyprshell_config_block;
use core_lib::transfer::ExternalTransferType;
use core_lib::{WarnWithDetails, notify_warn};
use relm4::adw::gdk::Display;
use relm4::adw::prelude::*;
use relm4::adw::{glib, gtk};
use relm4::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, style_context_add_provider_for_display,
};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, error, info, trace};
use windows_lib::overview::{OverviewRoot, OverviewRootInput};
use windows_lib::switch::{SwitchRoot, SwitchRootInput};

#[derive(Debug)]
pub struct Root {
    config: Box<config_lib::Config>,
    switch_root: Option<Controller<SwitchRoot>>,
    overview_root: Option<Controller<OverviewRoot>>,
    data_dir: Rc<PathBuf>,
    config_file: Rc<PathBuf>,
    css_path: Rc<PathBuf>,
    #[allow(unused)]
    cache_dir: Rc<PathBuf>,
}

#[derive(Debug)]
pub enum RootInput {
    OpenSwitch(core_lib::Direction),
    CloseSwitch(bool),
    OpenOverview,
    SetConfig(Box<config_lib::Config>),
    Reload,
}

#[derive(Debug)]
pub struct RootInit {
    pub external_event_receiver: Receiver<ExternalTransferType>,
    pub data_dir: Rc<PathBuf>,
    pub config_file: Rc<PathBuf>,
    pub css_path: Rc<PathBuf>,
    pub cache_dir: Rc<PathBuf>,
}

#[derive(Debug)]
pub enum RootOutput {}

#[relm4::component(pub)]
impl SimpleComponent for Root {
    type Init = RootInit;
    type Input = RootInput;
    type Output = RootOutput;

    view! {
        gtk::Window {
            set_title: Some("main-window")
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing Root");
        let model = Self {
            config: Box::from(config_lib::Config::default()),
            switch_root: None,
            overview_root: None,
            data_dir: init.data_dir,
            config_file: init.config_file,
            css_path: init.css_path,
            cache_dir: init.cache_dir,
        };
        let widgets = view_output!();
        let sender_clone = sender.clone();
        glib::spawn_future_local(async move {
            loop {
                let cause = init.external_event_receiver.recv().await;
                match cause {
                    Err(err) => {
                        error!("Failed to receive external event: {err:?}");
                        return;
                    }
                    Ok(msg) => handle_external(msg, &sender_clone),
                }
            }
        });

        sender.input_sender().emit(RootInput::Reload);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RootInput::OpenSwitch(dir) => {
                trace!("Opening switch, dir: {:?}", dir);
                if let Some(switch) = &self.switch_root {
                    switch.emit(SwitchRootInput::OpenSwitch(dir));
                }
            }
            RootInput::CloseSwitch(do_switch) => {
                trace!("Closing switch: {:?}", do_switch);
                if let Some(switch) = &self.switch_root {
                    switch.emit(SwitchRootInput::CloseSwitch(do_switch));
                }
            }
            RootInput::OpenOverview => {
                trace!("Opening overview");
                if let Some(overview) = &self.overview_root {
                    overview.emit(OverviewRootInput::OpenOverview);
                }
            }
            RootInput::Reload => self.load_config(&sender),
            RootInput::SetConfig(config) => {
                self.config = config;
                let app = relm4::main_application();
                let windows = app.windows();
                for window in windows {
                    if window.title() != Some("main-window".into()) {
                        window.close();
                    }
                }
                self.apply_css().warn_details("Failed to apply css");
                // force rebuild of windows
                let _ = self.overview_root.take();
                let _ = self.switch_root.take();
                self.update_switch();
                self.update_overview();
            }
        }
    }
}

fn handle_external(msg: ExternalTransferType, sender: &ComponentSender<Root>) {
    debug!("External event: {:?}", msg);
    match msg {
        ExternalTransferType::OpenOverview => {
            sender.input_sender().emit(RootInput::OpenOverview);
            gtk::gio::spawn_blocking(util::reload_desktop_data);
        }
        ExternalTransferType::OpenSwitch(cfg) => {
            sender
                .input_sender()
                .emit(RootInput::OpenSwitch(if cfg.reverse {
                    core_lib::Direction::Left
                } else {
                    core_lib::Direction::Right
                }));
        }
        ExternalTransferType::CloseSwitch(cfg) => {
            sender
                .input_sender()
                .emit(RootInput::CloseSwitch(cfg.switch));
        }
        ExternalTransferType::Reload => {
            sender.input_sender().emit(RootInput::Reload);
        }
    }
}

impl Root {
    fn update_overview(&mut self) {
        if let Some(windows) = &self.config.windows {
            if let Some(overview) = &windows.overview {
                if let Some(o_root) = &self.overview_root {
                    o_root.emit(OverviewRootInput::CloseOverview(false));
                    o_root.emit(OverviewRootInput::SetGeneral(windows.general.clone()));
                    o_root.emit(OverviewRootInput::SetOverview(overview.clone()));
                } else {
                    let app = relm4::main_application();

                    let overview_root = OverviewRoot::builder();
                    let window = &overview_root.root;
                    app.add_window(window);
                    let overview_root = overview_root
                        .launch(windows_lib::overview::OverviewRootInit {
                            general: windows.general.clone(),
                            overview: overview.clone(),
                            data_dir: self.data_dir.clone(),
                        })
                        .detach();
                    self.overview_root = Some(overview_root);
                }
            } else {
                let _ = self.overview_root.take();
            }
        } else {
            let _ = self.overview_root.take();
        }
    }

    fn update_switch(&mut self) {
        if let Some(windows) = &self.config.windows {
            if let Some(switch) = &windows.switch {
                if let Some(sw_root) = &self.switch_root {
                    sw_root.emit(SwitchRootInput::CloseSwitch(false));
                    sw_root.emit(SwitchRootInput::SetGeneral(windows.general.clone()));
                    sw_root.emit(SwitchRootInput::SetSwitch(switch.clone()));
                } else {
                    let app = relm4::main_application();

                    let switch_root = SwitchRoot::builder();
                    let window = &switch_root.root;
                    app.add_window(window);
                    let switch_root = switch_root
                        .launch(windows_lib::switch::SwitchRootInit {
                            general: windows.general.clone(),
                            switch: switch.clone(),
                        })
                        .detach();
                    self.switch_root = Some(switch_root);
                }
            } else {
                let _ = self.switch_root.take();
            }
        } else {
            let _ = self.switch_root.take();
        }
    }

    fn load_config(&self, sender: &ComponentSender<Self>) {
        let config = match config_lib::load_and_migrate_config(&self.config_file, true) {
            Ok(config) => config,
            Err(err) => {
                notify_warn(&format!(
                    "Failed to load config: {err:?}, retrying on change"
                ));
                if let Err(err) = hyprshell_config_block(&self.config_file) {
                    error!("Failed to block config: {err:?}");
                    notify_warn(&format!("Failed wait for config change: {err:?}"));
                }
                info!("Trying to reload config after config change");
                sender.input_sender().emit(RootInput::Reload);
                return;
            }
        };

        // TODO remove in future if more is available
        if config.windows.is_none()
            || matches!(&config.windows, Some(windows) if windows.overview.is_none() && windows.switch.is_none())
        {
            notify_warn("Nothing is enabled in the config, retrying on change");
            if let Err(err) = hyprshell_config_block(&self.config_file) {
                error!("Failed to block config: {err:?}");
                notify_warn(&format!("Failed wait for config change: {err:?}"));
            }
            info!("Trying to reload config after config change");
            sender.input_sender().emit(RootInput::Reload);
            return;
        }

        if let Err(err) = configure_wm(&config) {
            notify_warn(&format!(
                "Failed to configure wm: {err:?}, could not load config"
            ));
            return;
        }

        exec_lib::set_follow_mouse_default()
            .warn_details("Failed to set set_remain_focused default");
        sender
            .input_sender()
            .emit(RootInput::SetConfig(Box::from(config)));
    }

    fn apply_css(&self) -> anyhow::Result<()> {
        let provider_app = CssProvider::new();

        provider_app.load_from_string(include_str!("default_styles.css"));
        style_context_add_provider_for_display(
            &Display::default().context("Could not connect to a display.")?,
            &provider_app,
            STYLE_PROVIDER_PRIORITY_USER,
        );

        windows_lib::get_css()?;
        launcher_lib::get_css()?;

        if self.css_path.exists() {
            debug!("Loading custom css file {:?}", self.css_path);
            let provider_user = CssProvider::new();
            provider_user.load_from_path(&*self.css_path);
            style_context_add_provider_for_display(
                &Display::default().context("Could not connect to a display.")?,
                &provider_user,
                STYLE_PROVIDER_PRIORITY_USER,
            );
        } else {
            debug!("Custom css file {:?} does not exist", self.css_path);
        }
        Ok(())
    }
}
