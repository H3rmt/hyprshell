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
use tracing::trace;
use windows_lib::switch::{SwitchRoot, SwitchRootInput, SwitchRootOutput};

#[derive(Debug)]
pub struct Root {
    config: config_lib::Config,
    pub switch_root: Option<Controller<SwitchRoot>>,
}

#[derive(Debug)]
pub enum RootInput {
    SwitchClosed,
    OpenSwitch(core_lib::Direction),
    SwitchSwitch(core_lib::Direction),
    CloseSwitch(bool),
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
        };
        let widgets = view_output!();
        let sender_clone = sender.clone();
        glib::spawn_future_local(async move {
            loop {
                let cause = init.external_event_receiver.recv().await;
                match cause {
                    Err(err) => {
                        tracing::error!("Failed to receive external event: {err:?}");
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

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RootInput::SwitchClosed => {
                trace!("Switch closed");
            }
            RootInput::OpenSwitch(dir) => {
                trace!("Opening switch: {:?}", dir);
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
            RootInput::SetWindows(windows) => {
                self.config.windows = windows;
                self.update_switch(sender)
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
    match msg {
        ExternalTransferType::OpenOverview => {}
        ExternalTransferType::OpenSwitch(cfg) => {
            sender
                .input_sender()
                .emit(RootInput::OpenSwitch(if cfg.reverse {
                    core_lib::Direction::Left
                } else {
                    core_lib::Direction::Right
                }));
        }
        ExternalTransferType::CloseSwitch => {}
        ExternalTransferType::CloseAll => {}
        ExternalTransferType::Restart => {
            sender.input_sender().emit(RootInput::Restart);
        }
    }
}

impl Root {
    fn update_switch(&mut self, sender: ComponentSender<Self>) {
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
                        .forward(sender.input_sender(), |msg| match msg {
                            SwitchRootOutput::Closed => RootInput::SwitchClosed,
                        });
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
