use config_lib::Config;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
pub struct Root {
    config: config_lib::Config,
}

#[derive(Debug)]
pub enum RootInput {
    CreateLauncherWindow(),
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
        let model = Self {
            config: Config::default(),
        };

        let widgets = view_output!();

        sender.input(RootInput::CreateLauncherWindow());
        sender.input(RootInput::CreateLauncherWindow());
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RootInput::CreateLauncherWindow() => {
                let app = relm4::main_application();
                let builder = windows_lib::switch::SwitchRoot::builder();

                let window = &builder.root;
                app.add_window(window);

                window.set_visible(true);
                builder
                    .launch(windows_lib::switch::SwitchRootInit {
                        windows: self.config.windows.clone(),
                    })
                    .detach_runtime();
            }
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
