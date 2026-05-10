use async_channel::Receiver;
use core_lib::transfer::ExternalTransferType;
use exec_lib::listener::hyprland_config_listener;
use relm4::adw::prelude::*;
use relm4::adw::{glib, gtk};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use std::thread;
use std::time::Duration;
use tracing::{debug, trace};
use windows_lib::overview::{OverviewRoot, OverviewRootInput, OverviewRootOutput};
use windows_lib::switch::{SwitchRoot, SwitchRootInput, SwitchRootOutput};

#[derive(Debug)]
pub struct Root {
    config: config_lib::Config,
    pub switch_root: Option<Controller<SwitchRoot>>,
    pub overview_root: Option<Controller<OverviewRoot>>,
}

#[derive(Debug)]
pub enum RootInput {
    OpenSwitch(core_lib::Direction),
    SwitchSwitch(core_lib::Direction),
    CloseSwitch(bool),
    OpenOverview,
    SetWindows(Option<config_lib::Windows>),
    Restart,
}

#[derive(Debug)]
pub struct RootInit {
    pub config: config_lib::Config,
    pub external_event_receiver: Receiver<ExternalTransferType>,
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
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing Root");
        let model = Self {
            config: init.config,
            switch_root: None,
            overview_root: None,
        };
        let widgets = view_output!();
        let sender_clone = sender.clone();
        glib::spawn_future_local(async move {
            loop {
                let cause = init.external_event_receiver.recv().await;
                match cause {
                    Err(err) => {
                        tracing::error!("Failed to receive external event: {err:?}");
                        return;
                    }
                    Ok(msg) => handle_external(msg, &sender_clone),
                }
            }
        });

        sender
            .input_sender()
            .emit(RootInput::SetWindows(model.config.windows.clone()));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            RootInput::OpenSwitch(dir) => {
                trace!("Opening switch, dir: {:?}", dir);
                if let Some(switch) = &self.switch_root {
                    switch.emit(SwitchRootInput::OpenSwitch(dir))
                }
            }
            RootInput::SwitchSwitch(dir) => {
                trace!("Switching switch: {:?}", dir);
                if let Some(switch) = &self.switch_root {
                    switch.emit(SwitchRootInput::Switch(dir))
                }
            }
            RootInput::CloseSwitch(do_switch) => {
                trace!("Closing switch: {:?}", do_switch);
                if let Some(switch) = &self.switch_root {
                    switch.emit(SwitchRootInput::CloseSwitch(do_switch))
                }
            }
            RootInput::OpenOverview => {
                trace!("Opening overview");
                if let Some(overview) = &self.overview_root {
                    overview.emit(OverviewRootInput::OpenOverview)
                }
            }
            RootInput::SetWindows(windows) => {
                self.config.windows = windows;
                self.update_switch();
                self.update_overview();
            }
            RootInput::Restart => {
                let app = relm4::main_application();
                let mut windows = app.windows();
                for window in windows {
                    window.close()
                }
                thread::sleep(Duration::from_millis(250));
                app.quit();
            }
        }
    }
}

fn handle_external(msg: ExternalTransferType, sender: &ComponentSender<Root>) {
    debug!("External event: {:?}", msg);
    match msg {
        ExternalTransferType::OpenOverview => {
            sender.input_sender().emit(RootInput::OpenOverview);
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
        ExternalTransferType::Restart => {
            sender.input_sender().emit(RootInput::Restart);
        }
    }
}

impl Root {
    fn update_overview(&mut self) {
        if let Some(windows) = &self.config.windows {
            if let Some(overview) = &windows.overview {
                if let Some(o_root) = &self.overview_root {
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
}
