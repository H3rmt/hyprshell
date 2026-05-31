use crate::plugins;
use crate::plugins::{
    get_child_launch_items, get_input_driven_launch_items, get_launch_items,
    get_static_launch_items, get_static_options_chars, LaunchItem,
};
use crate::plugins_boxes::{LauncherPlugins, LauncherPluginsInit, LauncherPluginsOutput};
use crate::result::{LauncherResults, LauncherResultsInit, LauncherResultsOutput};
use config_lib::{Launcher, Modifier};
use core_lib::transfer::Identifier;
use core_lib::{Direction, LAUNCHER_NAMESPACE, WarnWithDetails};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::adw::gdk::ModifierType;
use relm4::adw::prelude::*;
use relm4::adw::{gdk, glib, gtk};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::{EventController, EventControllerKey, Orientation, PropagationPhase};
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
    plugins: FactoryVecDeque<LauncherPlugins>,
    controller: Option<EventController>,

    data: LauncherData,
    active_results: Vec<LauncherResultsInit>,
    active_parent: Option<LaunchItem>,
    child_text: Option<Box<str>>,
    child_cursor: Option<i32>,
    switching: bool,
    data_dir: Rc<PathBuf>,
}

#[derive(Debug)]
pub enum LauncherRootInput {
    SetLauncher(Launcher),
    OpenLauncher,
    CloseLauncher,
    Launch(char),
    LaunchIndex(usize),
    Escape,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivationOutcome {
    OpenChildMode,
    Launched,
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
                #[local_ref]
                pluginse -> gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_css_classes: &["launcher-plugins"],
                    set_spacing: 4,
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
            LauncherResultsOutput::Clicked(idx) => {
                LauncherRootInput::LaunchIndex(idx.current_index())
            }
            });
        let plugins: FactoryVecDeque<LauncherPlugins> = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |r| match r {
                LauncherPluginsOutput::Clicked(ch) => LauncherRootInput::Launch(ch),
            });

        let model = Self {
            launcher: init.launcher,
            data_dir: init.data_dir,
            window: root.clone(),
            entry,
            results,
            plugins,
            controller: None,
            data: LauncherData::default(),
            active_results: Vec::new(),
            active_parent: None,
            child_text: None,
            child_cursor: None,
            switching: false, // enter when nothing was done launches program
        };

        let entrye = &model.entry;
        let resultse = &model.results.widget().clone();
        let pluginse = &model.plugins.widget().clone();
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
                self.active_parent = None;
                self.child_text = None;
                self.child_cursor = None;
                self.switching = false;
                self.open_launcher();
                self.handle_type();
            }
            LauncherRootInput::CloseLauncher => self.close_launcher(),
            LauncherRootInput::Launch(char) => {
                trace!("Closing launcher with char: {}", char);
                if let Some(index) = char.to_digit(10).map(|a| a as usize) {
                    if self.active_parent.is_some() {
                        sender.input_sender().emit(LauncherRootInput::LaunchIndex(index));
                    } else {
                        match self.activate_selected(index) {
                            ActivationOutcome::OpenChildMode => {}
                            ActivationOutcome::Launched => {
                                sender
                                    .output_sender()
                                    .emit(LauncherRootOutput::Close(false));
                            }
                        }
                    }
                } else if let Some(iden) = self.data.static_matches.get(&char) {
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
            LauncherRootInput::LaunchIndex(index) => {
                trace!("Closing launcher with index: {}", index);
                match self.activate_selected(index) {
                    ActivationOutcome::OpenChildMode => return,
                    ActivationOutcome::Launched => {
                        sender
                            .output_sender()
                            .emit(LauncherRootOutput::Close(false));
                    }
                }
            }
            LauncherRootInput::Escape => {
                if self.active_parent.is_some() {
                    self.active_parent = None;
                    if let Some(child_text) = self.child_text.take() {
                        self.entry.set_text(&child_text);
                        if let Some(cursor) = self.child_cursor.take() {
                            self.entry.set_position(cursor);
                        } else {
                            self.entry.set_position(self.entry.text().len() as i32);
                        }
                    }
                    self.handle_type();
                } else {
                    sender.output_sender().emit(LauncherRootOutput::Close(false));
                }
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
                    sender.input_sender().emit(LauncherRootInput::LaunchIndex(0));
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
        self.active_parent = None;
        self.child_text = None;
        self.child_cursor = None;
        self.switching = false;
        self.window.set_visible(true);
        self.entry.grab_focus();
        self.entry.set_text("");
        exec_lib::set_no_follow_mouse(None).warn_details("Failed to set follow mouse");
    }
    fn close_launcher(&mut self) {
        trace!("Hiding window {:?}", self.window.id());
        self.window.set_visible(false);
        exec_lib::reset_no_follow_mouse().warn_details("Failed to reset follow mouse");
    }

    fn activate_selected(&mut self, index: usize) -> ActivationOutcome {
        let Some(opt) = self.active_results.get(index).map(|entry| &entry.opt) else {
            return ActivationOutcome::Launched;
        };

        if let Some(parent) = self.active_parent.as_ref() {
            if index == 0 {
                plugins::launch(
                    &parent.iden,
                    &self.entry.text(),
                    self.launcher.default_terminal.as_deref(),
                    &self.data_dir,
                );
                return ActivationOutcome::Launched;
            }

            if let Some(child) = parent.children.get(index - 1) {
                plugins::launch(
                    &child.iden,
                    &self.entry.text(),
                    self.launcher.default_terminal.as_deref(),
                    &self.data_dir,
                );
                return ActivationOutcome::Launched;
            }

            return ActivationOutcome::Launched;
        }

        if !opt.item.children.is_empty() {
            self.child_text = Some(self.entry.text().into());
            self.child_cursor = Some(self.entry.position());
            self.active_parent = Some(opt.item.clone());
            self.entry.set_text("");
            self.handle_type();
            return ActivationOutcome::OpenChildMode;
        }

        plugins::launch(
            &opt.item.iden,
            &self.entry.text(),
            self.launcher.default_terminal.as_deref(),
            &self.data_dir,
        );
        ActivationOutcome::Launched
    }

    fn handle_type(&mut self) {
        self.data.sorted_matches.clear();
        self.data.input_driven_matches.clear();
        self.data.static_matches.clear();
        let text: &str = &self.entry.text();

        let mut results_lock = self.results.guard();
        results_lock.clear();
        let mut plugins_lock = self.plugins.guard();
        plugins_lock.clear();

        if !self.launcher.show_when_empty && text.is_empty() {
            return;
        }
        let items = self.launcher.max_items.min(9) as usize;
        let mut results: Vec<LauncherResultsInit> = Vec::new();
        if let Some(parent) = self.active_parent.as_ref() {
            for (index, opt) in get_child_launch_items(parent, text)
                .into_iter()
                .take(items)
                .enumerate()
            {
                results.push(LauncherResultsInit {
                    opt,
                    key: match index {
                        0 => "Return".to_string(),
                        i => format!("{}+{i}", self.launcher.launch_modifier),
                    },
                    has_children: false,
                });
            }
        } else {
            for (index, opt) in get_launch_items(&self.launcher.plugins, text, &self.data_dir)
                .into_iter()
                .take(items)
                .enumerate()
            {
                self.data.sorted_matches.push(opt.item.iden.clone());
                let has_children = !opt.item.children.is_empty();
                results.push(LauncherResultsInit {
                    opt,
                    key: match index {
                        0 => "Return".to_string(),
                        i => format!("{}+{i}", self.launcher.launch_modifier),
                    },
                    has_children,
                });
            }

            for (index, opt) in get_input_driven_launch_items(&self.launcher.plugins, text)
                .into_iter()
                .take(items)
                .enumerate()
            {
                self.data.input_driven_matches.push(opt.item.iden.clone());
                results.push(LauncherResultsInit {
                    opt,
                    key: match index {
                        0 => "Return".to_string(),
                        i => format!("{}+{i}", self.launcher.launch_modifier),
                    },
                    has_children: false,
                });
            }
        }

        self.active_results = results.clone();
        for item in results {
            results_lock.push_back(item);
        }

        if self.active_parent.is_none() {
            for opt in get_static_launch_items(
                &self.launcher.plugins,
                self.launcher.default_terminal.as_deref(),
                text,
            ) {
                self.data
                    .static_matches
                    .entry(opt.key)
                    .or_insert(opt.iden.clone());
                plugins_lock.push_back(LauncherPluginsInit {
                    opt,
                    launch_modifier: self.launcher.launch_modifier,
                });
            }
        }
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
            sender.input_sender().emit(LauncherRootInput::Escape);
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
    pub sorted_matches: Vec<Identifier>,
    pub input_driven_matches: Vec<Identifier>,
    pub static_matches: HashMap<char, Identifier>,
}
