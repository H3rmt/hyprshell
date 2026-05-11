use crate::plugins;
use crate::plugins::{
    SortedLaunchOption, StaticLaunchOption, get_sorted_launch_options, get_static_launch_options,
    get_static_options_chars,
};
use crate::result::{LauncherResults, LauncherResultsInit, LauncherResultsOutput};
use config_lib::{Launcher, Modifier};
use core_lib::transfer::Identifier;
use core_lib::{Direction, LAUNCHER_NAMESPACE, WarnWithDetails};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::adw::gdk::ModifierType;
use relm4::adw::prelude::*;
use relm4::adw::{gdk, glib, gtk};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::{
    EventController, EventControllerKey, Orientation, PropagationPhase, SelectionMode,
};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{trace, warn};

#[derive(Debug)]
pub struct LauncherRoot {
    launcher: Launcher,
    window: gtk::ApplicationWindow,
    entry: gtk::Entry,
    results: FactoryVecDeque<LauncherResults>,
    controller: Option<EventController>,

    data: LauncherData,
    switching: bool,
    data_dir: Rc<PathBuf>,
    static_launch_options: Vec<StaticLaunchOption>,
    sortable_launch_options: Vec<SortedLaunchOption>,
}

#[derive(Debug)]
pub enum LauncherRootInput {
    SetLauncher(Launcher),
    OpenLauncher,
    CloseLauncher,
    Launch(char),
    Return,
    Switch(Direction, bool),
    Type,
}

#[derive(Debug)]
pub struct LauncherRootInit {
    pub launcher: Launcher,
    pub data_dir: Rc<PathBuf>,
}

#[derive(Debug)]
pub enum LauncherRootOutput {
    Switch(Direction, bool),
    Close(bool),
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
                set_spacing: 4,
                #[watch]
                set_width_request: i32::from(model.launcher.width),
                #[local_ref]
                entrye -> gtk::Entry {
                    set_css_classes: &["launcher-input"],
                    connect_changed => LauncherRootInput::Type,
                },
                #[local_ref]
                resultse -> gtk::Box {
                    set_orientation: Orientation::Vertical,
                    set_css_classes: &["launcher-results"],
                    set_spacing: 3,
                },
                // #[local_ref]
                // pluginse ->
                gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_css_classes: &["launcher-plugins"],
                    set_spacing: 4,
                    gtk::Label {
                        set_label: "Plugins, todo",
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let entry = gtk::Entry::new();
        let results: FactoryVecDeque<LauncherResults> = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |r| match r {
                LauncherResultsOutput::Clicked(idx) => LauncherRootInput::Launch(
                    idx.current_index()
                        .to_string()
                        .chars()
                        .next()
                        .expect("No char"),
                ),
            });

        let model = Self {
            launcher: init.launcher,
            data_dir: init.data_dir,
            window: root.clone(),
            entry,
            results,
            controller: None,
            data: LauncherData::default(),
            sortable_launch_options: vec![],
            static_launch_options: vec![],
            switching: false, // enter when nothing was done launches program
        };

        let entrye = &model.entry;
        let resultse = &model.results.widget().clone();
        let widgets = view_output!();

        // ensure that the entry is always focused
        let entry_2 = model.entry.clone();
        let window_2 = root.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if window_2.is_visible() {
                entry_2.grab_focus_without_selecting();
            }
            glib::ControlFlow::Continue
        });

        // TODO someday move to generic init fn
        plugins::init_calc_context();

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
            LauncherRootInput::OpenLauncher => {
                self.open_launcher();
                self.handle_type();
            }
            LauncherRootInput::CloseLauncher => self.close_launcher(),
            LauncherRootInput::Launch(char) => {
                trace!("Closing launcher with char: {}", char);
                if let Some(iden) = match char {
                    '0'..='9' => char
                        .to_digit(10)
                        .and_then(|a| self.data.sorted_matches.get(a as usize)),
                    _ => self.data.static_matches.get(&char),
                } {
                    plugins::launch(
                        iden,
                        &self.entry.text(),
                        self.launcher.default_terminal.as_deref(),
                        &self.data_dir,
                    );
                } else {
                    warn!("No match found for char: {}", char);
                }

                sender
                    .output_sender()
                    .emit(LauncherRootOutput::Close(false));
            }
            LauncherRootInput::Type => {
                self.switching = false;
                self.handle_type()
            }
            LauncherRootInput::Switch(dir, ws) => {
                self.switching = true;
                sender
                    .output_sender()
                    .emit(LauncherRootOutput::Switch(dir, ws));
            }
            LauncherRootInput::Return => {
                if !self.switching {
                    sender.input_sender().emit(LauncherRootInput::Launch('0'));
                } else {
                    sender.output_sender().emit(LauncherRootOutput::Close(true));
                }
            }
        }
    }
}

impl LauncherRoot {
    fn setup_keyboard_controller(&mut self, sender: &ComponentSender<Self>) {
        let event_controller = EventControllerKey::new();
        let plugin_keys = get_static_options_chars(&self.launcher.plugins);
        let sender_2 = sender.clone();
        let launcher = self.launcher.clone();
        let entry = self.entry.clone();
        event_controller.set_propagation_phase(PropagationPhase::Capture);
        event_controller.connect_key_pressed(move |_, key, _, modt| {
            trace!("input: {key:?}");
            let text_empty = entry.text().is_empty();
            handle_key(
                &launcher,
                text_empty,
                key,
                modt,
                &plugin_keys,
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
        self.entry.set_text("");
        exec_lib::set_no_follow_mouse().warn_details("Failed to set follow mouse");
    }
    fn close_launcher(&mut self) {
        trace!("Hiding window {:?}", self.window.id());
        self.window.set_visible(false);
        exec_lib::reset_no_follow_mouse().warn_details("Failed to reset follow mouse");
    }

    fn handle_type(&mut self) {
        let mut results_lock = self.results.guard();
        results_lock.clear();

        self.data.sorted_matches.clear();
        self.data.static_matches.clear();

        let text: &str = &self.entry.text();
        if !self.launcher.show_when_empty && text.is_empty() {
            return;
        }
        let items = self.launcher.max_items.min(9) as usize;
        trace!("update");

        for (index, (_, opt)) in
            get_sorted_launch_options(&self.launcher.plugins, text, &self.data_dir)
                .into_iter()
                .take(items)
                .enumerate()
        {
            self.data.sorted_matches.push(opt.iden.clone());
            results_lock.push_back(LauncherResultsInit {
                opt,
                key: match index {
                    0 => "Return".to_string(),
                    i => format!("{}+{i}", self.launcher.launch_modifier),
                },
            });
        }

        drop(results_lock)
        // self.static_launch_options = get_static_launch_options(
        //     &self.launcher.plugins,
        //     self.launcher.default_terminal.as_deref(),
        // );
    }
}

fn handle_key(
    launcher: &Launcher,
    text_empty: bool,
    key: gdk::Key,
    modt: ModifierType,
    plugin_keys: &[gdk::Key],
    sender: ComponentSender<LauncherRoot>,
) -> glib::Propagation {
    let launch_mod = match launcher.launch_modifier {
        Modifier::Ctrl => modt == ModifierType::CONTROL_MASK,
        Modifier::Alt => modt == ModifierType::ALT_MASK,
        Modifier::Super => modt == ModifierType::SUPER_MASK,
        Modifier::None => false,
    };
    trace!(
        "key: {}{:?}, mods: {:?}, launch_mod: {}, launch_modifier: {}",
        key, key, modt, launch_mod, launcher.launch_modifier
    );
    if launch_mod && plugin_keys.contains(&key) {
        if let Some(ch) = key.name().unwrap_or_default().to_string().pop() {
            sender.input_sender().emit(LauncherRootInput::Launch(ch));
        }
        return glib::Propagation::Stop;
    }

    match (launch_mod, key) {
        (_, gdk::Key::Escape) => {
            sender
                .output_sender()
                .emit(LauncherRootOutput::Close(false));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Tab) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Right, false));
            glib::Propagation::Stop
        }
        (_, gdk::Key::ISO_Left_Tab | gdk::Key::grave | gdk::Key::dead_grave) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Left, false));
            glib::Propagation::Stop
        }
        (true, gdk::Key::h) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Left, true));
            glib::Propagation::Stop
        }
        (true, gdk::Key::l) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Right, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Left) => {
            if !text_empty {
                // allow using with text in launcher
                return glib::Propagation::Proceed;
            }
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Left, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Right) => {
            if !text_empty {
                // allow using with text in launcher
                return glib::Propagation::Proceed;
            }
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Right, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Up) | (true, gdk::Key::k) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Up, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Down) | (true, gdk::Key::j) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::Switch(Direction::Down, true));
            glib::Propagation::Stop
        }
        (_, gdk::Key::Return) => {
            sender.input_sender().emit(LauncherRootInput::Return);
            glib::Propagation::Stop
        }
        (true, gdk::Key::_1) => {
            sender.input_sender().emit(LauncherRootInput::Launch('1'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_2) => {
            sender.input_sender().emit(LauncherRootInput::Launch('2'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_3) => {
            sender.input_sender().emit(LauncherRootInput::Launch('3'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_4) => {
            sender.input_sender().emit(LauncherRootInput::Launch('4'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_5) => {
            sender.input_sender().emit(LauncherRootInput::Launch('5'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_6) => {
            sender.input_sender().emit(LauncherRootInput::Launch('6'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_7) => {
            sender.input_sender().emit(LauncherRootInput::Launch('7'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_8) => {
            sender.input_sender().emit(LauncherRootInput::Launch('8'));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_9) => {
            sender.input_sender().emit(LauncherRootInput::Launch('9'));
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    }
}

#[derive(Debug, Default)]
pub struct LauncherData {
    pub results_items: HashMap<Identifier, (gtk::Box, HashMap<Identifier, gtk::ListBoxRow>)>,
    pub plugins_items: HashMap<Identifier, gtk::Button>,

    pub sorted_matches: Vec<Identifier>,
    pub static_matches: HashMap<char, Identifier>,
}
