use relm4::adw::gtk;
use relm4::adw::prelude::*;
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
}

#[derive(Debug)]
pub struct RootInit {}

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
        let mut conf = config_lib::Config::default();
        let mut windows = config_lib::Windows::default();
        let mut switch = config_lib::Switch::default();
        switch.switch_workspaces = true;
        windows.switch = Some(switch);
        conf.windows = Some(windows);

        let model = Self {
            config: conf,
            switch_root: None,
        };
        let widgets = view_output!();

        // TODO remove
        let sender_clone = sender.input_sender().clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            sender_clone
                .send(RootInput::OpenSwitch(core_lib::Direction::Right))
                .ok();
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

// if env::var_os("HYPRSHELL_NO_LISTENERS").is_none() {
//     register_event_restarter(config_file.clone(), css_path.clone(), event_sender.clone());
// }
//
// let event_sender_2 = event_sender.clone();
// let event_receiver_2 = event_receiver.clone();
// thread::spawn(move || {
//     socket_handler(event_sender_2, event_receiver_2);
// });
