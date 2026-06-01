use crate::plugin::{
    LaunchItem, MatchedLaunchItem, PluginItem, get_child_launch_items_from_parent,
    match_launch_item,
};
use crate::plugins;
use crate::plugins_boxes::{
    LauncherPlugins, LauncherPluginsInit, LauncherPluginsInput, LauncherPluginsOutput,
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
use relm4::gtk::{EventController, EventControllerKey, Orientation, PropagationPhase};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{trace, warn};

#[derive(Debug)]
pub struct LauncherRoot {
    settings: Launcher,

    ui: LauncherUI,
    data: LauncherData,

    switching: bool,
    data_dir: Rc<PathBuf>,
}

#[derive(Debug)]
pub enum LauncherRootInput {
    SetLauncher(Launcher),
    OpenLauncher,
    CloseLauncher,
    LaunchPlugin(char),
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
    /// do_switch: if true, opens program / does switch and closes, if false only closes
    Close(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivationOutcome {
    OpenChildMode,
    Launched,
    NotLaunched,
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
                set_width_request: i32::from(model.settings.width),
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
                LauncherPluginsOutput::Clicked(ch) => LauncherRootInput::LaunchPlugin(ch),
            });

        let model = Self {
            settings: init.launcher,
            data_dir: init.data_dir,
            ui: LauncherUI {
                window: root.clone(),
                entry,
                results,
                plugins,
                controller: None,
            },
            data: LauncherData::default(),
            switching: false, // enter when nothing was done launches program
        };

        let entrye = &model.ui.entry;
        let resultse = &model.ui.results.widget().clone();
        let pluginse = &model.ui.plugins.widget().clone();
        let widgets = view_output!();

        // ensure that the entry is always focused
        let entry_2 = model.ui.entry.clone();
        let window_2 = root.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if window_2.is_visible() {
                entry_2.grab_focus_without_selecting();
            }
            glib::ControlFlow::Continue
        });
        plugins::init();

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
                self.settings = launcher;
                self.setup_keyboard_controller(&sender);
            }
            LauncherRootInput::OpenLauncher => {
                self.reset_data();
                self.load_static_items();
                self.load_static_plugins();
                self.handle_type();
                self.open_launcher();
            }
            LauncherRootInput::CloseLauncher => self.close_launcher(),
            LauncherRootInput::LaunchPlugin(char) => {
                trace!("Closing launcher with char: {}", char);
                if let Some(iden) = self.data.static_plugins.get(&char) {
                    plugins::launch(
                        iden,
                        &self.ui.entry.text(),
                        self.settings.default_terminal.as_deref(),
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
                    ActivationOutcome::NotLaunched => {}
                }
            }
            LauncherRootInput::Escape => {
                if self.data.active_parent.is_some() {
                    self.data.active_parent = None;
                    if let Some(text) = self.data.parent_text.take()
                        && let Some(cursor) = self.data.parent_cursor.take()
                    {
                        self.ui.entry.set_text(&text);
                        self.ui.entry.set_position(cursor);
                    }
                    self.handle_type();
                } else {
                    sender
                        .output_sender()
                        .emit(LauncherRootOutput::Close(false));
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
                    sender
                        .input_sender()
                        .emit(LauncherRootInput::LaunchIndex(0));
                } else {
                    sender.output_sender().emit(LauncherRootOutput::Close(true));
                }
            }
        }
    }
}

impl LauncherRoot {
    fn reset_data(&mut self) {
        self.data.active_parent = None;
        self.data.parent_text = None;
        self.data.parent_cursor = None;
        self.data.active_results.clear();
        self.switching = false;
    }

    fn load_static_items(&mut self) {
        for opt in plugins::get_static_items(&self.settings.plugins, &self.data_dir) {
            self.data.static_items.push(opt);
        }
    }

    fn load_static_plugins(&mut self) {
        let plugins = plugins::get_static_plugins(
            &self.settings.plugins,
            self.settings.default_terminal.as_deref(),
        );
        let mut plugins_lock = self.ui.plugins.guard();
        plugins_lock.clear();
        for opt in plugins {
            self.data.static_plugins.insert(opt.key, opt.iden.clone());
            plugins_lock.push_back(LauncherPluginsInit {
                opt,
                launch_modifier: self.settings.launch_modifier,
            });
        }
    }

    fn setup_keyboard_controller(&mut self, sender: &ComponentSender<Self>) {
        let event_controller = EventControllerKey::new();
        let plugin_keys = plugins::get_static_options_chars(&self.settings.plugins);
        let sender_2 = sender.clone();
        let launcher = self.settings.clone();
        let entry = self.ui.entry.clone();
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
        if let Some(controller) = self.ui.controller.take() {
            self.ui.entry.remove_controller(&controller);
        }
        self.ui.entry.add_controller(event_controller);
    }
    fn open_launcher(&mut self) {
        trace!("Showing window {:?}", self.ui.window.id());
        self.ui.window.set_visible(true);
        self.ui.entry.grab_focus();
        self.ui.entry.set_text("");
        exec_lib::set_no_follow_mouse(None).warn_details("Failed to set follow mouse");
    }
    fn close_launcher(&mut self) {
        trace!("Hiding window {:?}", self.ui.window.id());
        self.ui.window.set_visible(false);
        exec_lib::reset_no_follow_mouse().warn_details("Failed to reset follow mouse");
    }

    fn activate_selected(&mut self, index: usize) -> ActivationOutcome {
        let Some(item) = self.data.active_results.get(index).map(|entry| &entry.item) else {
            return ActivationOutcome::NotLaunched;
        };
        if item.item.children.is_empty() {
            plugins::launch(
                &item.item.iden,
                &self.ui.entry.text(),
                self.settings.default_terminal.as_deref(),
                &self.data_dir,
            );
            ActivationOutcome::Launched
        } else {
            self.data.parent_text = Some(self.ui.entry.text().into());
            self.data.parent_cursor = Some(self.ui.entry.position());
            self.data.active_parent = Some(item.item.clone());
            self.ui.entry.set_text("");
            self.handle_type();
            ActivationOutcome::OpenChildMode
        }
    }

    fn handle_type(&mut self) {
        let text: &str = &self.ui.entry.text();

        let mut dynamic_results = Vec::new();
        let mut results = Vec::new();
        if !text.is_empty() || self.settings.show_when_empty {
            if let Some(parent) = self.data.active_parent.as_ref() {
                for opt in get_child_launch_items_from_parent(parent) {
                    results.push(opt);
                }
            } else {
                if !text.is_empty() {
                    for opt in plugins::get_input_driven_launch_items(&self.settings.plugins, text)
                    {
                        dynamic_results.push(opt);
                    }
                }
                results.extend(self.data.static_items.clone())
            }
        }

        let mut results: Vec<_> = results
            .into_iter()
            .filter_map(|item| match_launch_item(item, text))
            .collect();
        // reverse sorting, so that the most relevant items are at the top
        results.sort_by(|a, b| b.score.cmp(&a.score));
        dynamic_results.extend(results);

        let max_items = self.settings.max_items.min(9) as usize;
        let dynamic: Vec<_> = dynamic_results
            .into_iter()
            .enumerate()
            .map(|(idx, item)| LauncherResultsInit {
                has_children: !item.item.children.is_empty(),
                item,
                key: match idx {
                    0 => "Return".to_string(),
                    i => format!("{}+{i}", self.settings.launch_modifier),
                },
            })
            .take(max_items)
            .collect();

        self.data.active_results = dynamic.clone();
        let mut results_lock = self.ui.results.guard();
        results_lock.clear();
        for item in dynamic {
            results_lock.push_back(item);
        }

        self.ui
            .plugins
            .broadcast(LauncherPluginsInput::SetEnabled(!text.is_empty()))
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
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchPlugin(ch));
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
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(1));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_2) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(2));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_3) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(3));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_4) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(4));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_5) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(5));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_6) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(6));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_7) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(7));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_8) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(8));
            glib::Propagation::Stop
        }
        (true, gdk::Key::_9) => {
            sender
                .input_sender()
                .emit(LauncherRootInput::LaunchIndex(9));
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    }
}

#[derive(Debug, Default)]
struct LauncherData {
    static_items: Vec<LaunchItem>,
    static_plugins: HashMap<char, Identifier>,

    active_results: Vec<LauncherResultsInit>,

    active_parent: Option<LaunchItem>,
    parent_text: Option<Box<str>>,
    parent_cursor: Option<i32>,
}

#[derive(Debug)]
struct LauncherUI {
    window: gtk::ApplicationWindow,
    entry: gtk::Entry,
    results: FactoryVecDeque<LauncherResults>,
    plugins: FactoryVecDeque<LauncherPlugins>,
    controller: Option<EventController>,
}
