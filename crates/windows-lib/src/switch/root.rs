use crate::data::{SortConfig, collect_data};
use crate::next::{find_next_client, find_next_workspace};
use crate::switch::workspaces::{Items, ItemsInit, ItemsInput};
use core_lib::{Active, ClientId, HyprlandData, SWITCH_NAMESPACE, WorkspaceId};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::gtk::gdk::Key;
use relm4::gtk::{EventControllerKey, Orientation, SelectionMode};
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::trace;

#[derive(Debug)]
pub struct SwitchRoot {
    windows: Option<config_lib::Windows>,
    window: gtk::ApplicationWindow,
    controller: Option<gtk::EventController>,
    remove_html: Regex,
    data: SwitchData,
    items: FactoryVecDeque<Items>,
}

#[derive(Debug)]
pub enum SwitchRootInput {
    SetWindows(Option<config_lib::Windows>),
    OpenSwitch(core_lib::Direction),
}

#[derive(Debug)]
pub struct SwitchRootInit {
    pub windows: Option<config_lib::Windows>,
}

#[derive(Debug)]
pub enum SwitchRootOutput {
    CloseSwitch,
    Switch(core_lib::Direction),
    CloseItem,
}

#[relm4::component(pub)]
impl SimpleComponent for SwitchRoot {
    type Init = SwitchRootInit;
    type Input = SwitchRootInput;
    type Output = SwitchRootOutput;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_css_classes: &["window"],
            set_default_size: (100, 100),
            #[watch]
            set_visible: model.windows.is_some(),
            match &model.windows {
                None => gtk::FlowBox {},
                Some(windows) => {
                    #[local_ref]
                    itemsw -> gtk::FlowBox {
                        set_selection_mode: SelectionMode::None,
                        set_orientation: Orientation::Horizontal,
                        #[watch]
                        set_max_children_per_line: u32::from(windows.items_per_row),
                        #[watch]
                        set_min_children_per_line: u32::from(windows.items_per_row),
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
        let mut items = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .detach();

        let model = Self {
            windows: init.windows,
            window: root.clone(),
            controller: None,
            remove_html: Regex::new(r"<[^>]*>").expect("invalid regex"),
            data: SwitchData::default(),
            items,
        };

        let itemsw = model.items.widget();
        let widgets = view_output!();

        let window = &root;
        window.init_layer_shell();
        window.set_namespace(Some(SWITCH_NAMESPACE));
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, 430i32);
        window.set_keyboard_mode(KeyboardMode::OnDemand);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SwitchRootInput::SetWindows(windows) => {
                self.windows = windows;

                if let Some(windows) = &self.windows {
                    if let Some(switch) = &windows.switch {
                        // TODO add a check in config check so these always succeed
                        let s_key = Key::from_name(switch.key.to_string()).unwrap();
                        let kill_key = Key::from_name(switch.kill_key.to_string()).unwrap();
                        let key_controller = EventControllerKey::new();
                        let sender_2 = sender.clone();
                        key_controller.connect_key_pressed(move |_, key, _, _| {
                            handle_key(key, s_key, kill_key, sender_2.clone())
                        });
                        let r#mod = switch.modifier;
                        let sender_2 = sender.clone();
                        key_controller.connect_key_released(move |_, key, _, _| {
                            handle_release(key, r#mod, sender_2.clone());
                        });
                        if let Some(controller) = self.controller.take() {
                            self.window.remove_controller(&controller);
                        }
                        self.window.add_controller(key_controller);
                    }
                }
            }
            SwitchRootInput::OpenSwitch(open) => {
                if let Some(windows) = &self.windows {
                    if let Some(switch) = &windows.switch {
                        let (clients_data, active_prev) = match collect_data(&SortConfig {
                            filter_current_monitor: switch
                                .filter_by
                                .contains(&config_lib::FilterBy::CurrentMonitor),
                            filter_current_workspace: switch
                                .filter_by
                                .contains(&config_lib::FilterBy::CurrentWorkspace),
                            filter_same_class: switch
                                .filter_by
                                .contains(&config_lib::FilterBy::SameClass),
                            sort_recent: true,
                            exclude_workspaces: if switch.exclude_workspaces.is_empty() {
                                None
                            } else {
                                Some(switch.exclude_workspaces.clone())
                            },
                        }) {
                            Ok(data) => data,
                            Err(e) => {
                                tracing::error!("Failed to collect data: {}", e);
                                return;
                            }
                        };

                        let active = if switch.switch_workspaces {
                            find_next_workspace(
                                &open,
                                true,
                                &clients_data,
                                active_prev,
                                windows.items_per_row,
                            )
                        } else {
                            find_next_client(
                                &open,
                                true,
                                &clients_data,
                                active_prev,
                                windows.items_per_row,
                            )
                        };
                        self.window.id();
                        trace!("Showing window {:?}", self.window.id());
                        self.window.set_visible(true);
                        self.data = SwitchData {
                            active,
                            // hypr_data: clients_data,
                        };
                        let mut lock = self.items.guard();
                        lock.clear();
                        for (id, data) in clients_data.workspaces.into_iter() {
                            lock.push_back(ItemsInit {
                                remove_html: self.remove_html.clone(),
                                id,
                                data,
                                scale: windows.scale,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn handle_release(
    key: Key,
    modifier: config_lib::Modifier,
    event_sender: ComponentSender<SwitchRoot>,
) {
    if ((key == Key::Alt_L || key == Key::Alt_R) && modifier == config_lib::Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R)
            && modifier == config_lib::Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && modifier == config_lib::Modifier::Super)
    {
        event_sender
            .output_sender()
            .emit(SwitchRootOutput::CloseSwitch);
    }
}

fn handle_key(
    key: Key,
    s_key: Key,
    kill_key: Key,
    event_sender: ComponentSender<SwitchRoot>,
) -> gtk::glib::Propagation {
    match key {
        Key::Escape => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::CloseSwitch);
            gtk::glib::Propagation::Stop
        }
        k if k == s_key || k == Key::l || k == Key::Right => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::Switch(core_lib::Direction::Right));
            gtk::glib::Propagation::Stop
        }
        Key::ISO_Left_Tab | Key::grave | Key::dead_grave | Key::h | Key::Left => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::Switch(core_lib::Direction::Left));
            gtk::glib::Propagation::Stop
        }
        Key::j | Key::Down => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::Switch(core_lib::Direction::Down));
            gtk::glib::Propagation::Stop
        }
        Key::k | Key::Up => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::Switch(core_lib::Direction::Up));
            gtk::glib::Propagation::Stop
        }
        k if k == kill_key || k == Key::Delete => {
            event_sender
                .output_sender()
                .emit(SwitchRootOutput::CloseItem);
            gtk::glib::Propagation::Stop
        }
        _ => gtk::glib::Propagation::Proceed,
    }
}

#[derive(Debug)]
pub struct SwitchData {
    pub active: Active,
    // pub hypr_data: HyprlandData,
}

impl Default for SwitchData {
    fn default() -> Self {
        Self {
            active: Active {
                client: None,
                workspace: -1,
                monitor: -1,
            },
            // hypr_data: HyprlandData::default(),
        }
    }
}
