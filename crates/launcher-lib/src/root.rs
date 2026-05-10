use crate::plugins::get_static_options_chars;
use config_lib::{Launcher, Modifier};
use core_lib::{Direction, LAUNCHER_NAMESPACE};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::adw::gdk::ModifierType;
use relm4::adw::prelude::*;
use relm4::adw::{gdk, glib, gtk};
use relm4::gtk::{
    EventController, EventControllerKey, Orientation, PropagationPhase, SelectionMode,
};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use tracing::trace;

#[derive(Debug)]
pub struct LauncherRoot {
    launcher: Launcher,
    window: gtk::ApplicationWindow,
    entry: gtk::Entry,
    results: gtk::Box,
    plugins: gtk::Box,
    controller: Option<EventController>,
}

#[derive(Debug)]
pub enum LauncherRootInput {
    SetLauncher(Launcher),
    OpenLauncher,
    CloseLauncher,
    Launch(char),
}

#[derive(Debug)]
pub struct LauncherRootInit {
    pub launcher: Launcher,
}

#[derive(Debug)]
pub enum LauncherRootOutput {
    Switch(Direction, bool),
    Close,
}

#[relm4::component(pub)]
impl SimpleComponent for LauncherRoot {
    type Init = LauncherRootInit;
    type Input = LauncherRootInput;
    type Output = LauncherRootOutput;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_css_classes: &["window"],
            set_default_size: (20, 20),
            gtk::Box {
                set_css_classes: &["launcher"],
                set_orientation: Orientation::Vertical,
                #[watch]
                set_width_request: i32::from(model.launcher.width),
                #[local_ref]
                entrye -> gtk::Entry {
                    set_css_classes: &["launcher-input"],
                },
                #[local_ref]
                resultse -> gtk::Box {
                    set_orientation: Orientation::Vertical,
                    set_css_classes: &["launcher-results"],
                    set_spacing: 3,
                },
                #[local_ref]
                pluginse -> gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_css_classes: &["launcher-plugins"],
                    set_spacing: 4,
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let entry = gtk::Entry::new();
        let results = gtk::Box::default();
        let plugins = gtk::Box::default();

        let model = Self {
            launcher: init.launcher,
            window: root.clone(),
            entry,
            results,
            plugins,
            controller: None,
        };

        let entrye = &model.entry;
        let resultse = &model.results;
        let pluginse = &model.plugins;
        let widgets = view_output!();

        let window = &root;
        window.init_layer_shell();
        window.set_namespace(Some(LAUNCHER_NAMESPACE));
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, 0);
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LauncherRootInput::SetLauncher(launcher) => {
                self.launcher = launcher;
                self.setup_keyboard_controller(&sender);
            }
            LauncherRootInput::OpenLauncher => self.open_launcher(),
            LauncherRootInput::CloseLauncher => self.close_launcher(),
            LauncherRootInput::Launch(chr) => {
                todo!()
            }
        }
    }
}

impl LauncherRoot {
    fn setup_keyboard_controller(&mut self, sender: &ComponentSender<Self>) {
        let event_controller = EventControllerKey::new();
        let plugin_keys = get_static_options_chars(&self.launcher.plugins);
        let entry = self.entry.clone();
        let results = self.results.clone();
        let launch_modifier = self.launcher.launch_modifier;
        let sender_2 = sender.clone();
        event_controller.set_propagation_phase(PropagationPhase::Capture);
        event_controller.connect_key_pressed(move |_, key, _, modt| {
            trace!("input: {key:?}");
            handle_key(
                &entry,
                key,
                modt,
                &plugin_keys,
                launch_modifier,
                &results,
                sender_2.clone(),
            )
        });
        if let Some(controller) = self.controller.take() {
            self.entry.remove_controller(&controller);
        }
        self.entry.add_controller(event_controller);
    }
    fn open_launcher(&mut self) {
        trace!("Showing window {:?}", self.window.id());
        self.window.set_visible(true);
        self.entry.grab_focus();
    }
    fn close_launcher(&mut self) {
        trace!("Hiding window {:?}", self.window.id());
        self.window.set_visible(false);
    }
}

fn handle_key(
    entry: &gtk::Entry,
    key: gdk::Key,
    modt: ModifierType,
    plugin_keys: &[gdk::Key],
    launch_modifier: Modifier,
    results: &gtk::Box,
    sender: ComponentSender<LauncherRoot>,
) -> glib::Propagation {
    let launch_mod = match launch_modifier {
        Modifier::Ctrl => modt == ModifierType::CONTROL_MASK,
        Modifier::Alt => modt == ModifierType::ALT_MASK,
        Modifier::Super => modt == ModifierType::SUPER_MASK,
        Modifier::None => false,
    };
    trace!(
        "key: {}{:?}, mods: {:?}, launch_mod: {}, launch_modifier: {}",
        key, key, modt, launch_mod, launch_modifier
    );
    if launch_mod && plugin_keys.contains(&key) {
        if let Some(ch) = key.name().unwrap_or_default().to_string().pop() {
            sender.input_sender().emit(LauncherRootInput::Launch(ch));
        }
        return glib::Propagation::Stop;
    }

    match (launch_mod, key) {
        (_, gdk::Key::Escape) => {
            sender.output_sender().emit(LauncherRootOutput::Close);
            glib::Propagation::Stop
        }
        (_, gdk::Key::Tab) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Right, false));
            glib::Propagation::Stop
        }
        (_, gdk::Key::ISO_Left_Tab | gdk::Key::grave | gdk::Key::dead_grave) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Left, false));
            glib::Propagation::Stop
        }
        (true, gdk::Key::h) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Left, true));
            glib::Propagation::Stop
        }
        (true, gdk::Key::l) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Right, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Left) => {
            if !entry.text().is_empty() {
                // allow using with text in launcher
                return glib::Propagation::Proceed;
            }
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Left, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Right) => {
            if !entry.text().is_empty() {
                // allow using with text in launcher
                return glib::Propagation::Proceed;
            }
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Right, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Up) | (true, gdk::Key::k) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Up, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Down) | (true, gdk::Key::j) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Switch(Direction::Down, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Return) => {
            // TODO
            // if results.first_child().is_some() {
            //     event_sender
            //         .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::None))
            //         .warn_details("unable to send");
            // }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_1) => {
            if results.observe_children().into_iter().len() > 1 {
                sender.input_sender().emit(LauncherRootInput::Launch('1'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_2) => {
            if results.observe_children().into_iter().len() > 2 {
                sender.input_sender().emit(LauncherRootInput::Launch('2'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_3) => {
            if results.observe_children().into_iter().len() > 3 {
                sender.input_sender().emit(LauncherRootInput::Launch('3'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_4) => {
            if results.observe_children().into_iter().len() > 4 {
                sender.input_sender().emit(LauncherRootInput::Launch('4'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_5) => {
            if results.observe_children().into_iter().len() > 5 {
                sender.input_sender().emit(LauncherRootInput::Launch('5'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_6) => {
            if results.observe_children().into_iter().len() > 6 {
                sender.input_sender().emit(LauncherRootInput::Launch('6'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_7) => {
            if results.observe_children().into_iter().len() > 7 {
                sender.input_sender().emit(LauncherRootInput::Launch('7'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_8) => {
            if results.observe_children().into_iter().len() > 8 {
                sender.input_sender().emit(LauncherRootInput::Launch('8'));
            }
            glib::Propagation::Stop
        }
        (true, gdk::Key::_9) => {
            if results.observe_children().into_iter().len() > 9 {
                sender.input_sender().emit(LauncherRootInput::Launch('9'));
            }
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    }
}
