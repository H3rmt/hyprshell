use crate::shared::{Workspaces, WorkspacesInit, WorkspacesInput, WorkspacesOutput};
use core_lib::{
    Active, ClientData, ClientId, MonitorData, OVERVIEW_NAMESPACE, WorkspaceData, WorkspaceId,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
#[cfg(feature = "live_windows")]
use relm4::adw::gtk::glib;
use relm4::adw::prelude::*;
use relm4::adw::{gdk, gtk};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::{Orientation, SelectionMode};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
#[cfg(feature = "live_windows")]
use std::cell::RefCell;
#[cfg(feature = "live_windows")]
use std::rc::Rc;
use tracing::trace;

#[derive(Debug)]
pub struct OverviewWindow {
    general: config_lib::WindowsGeneral,
    open: bool,
    // gtk
    window: gtk::ApplicationWindow,
    /// Regex for removing HTML tags from strings
    remove_html: Regex,
    /// Factory for workspaces
    items: FactoryVecDeque<Workspaces>,
    #[cfg(feature = "live_windows")]
    live_thumbnails: bool,
    #[cfg(feature = "live_windows")]
    live_thumbnails_icons: bool,
    #[cfg(feature = "live_windows")]
    after_paint_clock: Option<gdk::FrameClock>,
    #[cfg(feature = "live_windows")]
    after_paint_handler: Rc<RefCell<Option<glib::SignalHandlerId>>>,
}

#[derive(Debug)]
pub enum OverviewWindowInput {
    SetGeneral(config_lib::WindowsGeneral),
    OpenOverview((OverviewWindowData, u16)),
    CloseOverview,
    ReloadOverview(OverviewWindowData),
    SetActive(Active, Active),
    #[cfg(feature = "live_windows")]
    UpdateClientThumbnail(ClientId, gdk::Texture),
}

#[derive(Debug)]
pub struct OverviewWindowInit {
    pub general: config_lib::WindowsGeneral,
    pub gtk_monitor: gdk::Monitor,
    #[cfg(feature = "live_windows")]
    pub live_thumbnails: bool,
    #[cfg(feature = "live_windows")]
    pub live_thumbnails_icons: bool,
}

#[derive(Debug)]
pub enum OverviewWindowOutput {
    Clicked(WorkspaceId),
    ClickedC(ClientId),
    #[cfg(feature = "live_windows")]
    Hidden,
}

#[relm4::component(pub)]
impl SimpleComponent for OverviewWindow {
    type Init = OverviewWindowInit;
    type Input = OverviewWindowInput;
    type Output = OverviewWindowOutput;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_css_classes: &["window"],
            set_default_size: (100, 100),
            #[local_ref]
            itemsw -> gtk::FlowBox {
                set_css_classes: &["monitor"],
                set_selection_mode: SelectionMode::None,
                set_orientation: Orientation::Horizontal,
                #[watch]
                set_max_children_per_line: u32::from(model.general.items_per_row),
                #[watch]
                set_min_children_per_line: u32::from(model.general.items_per_row),
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing OverviewWindow");

        let items: FactoryVecDeque<Workspaces> = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.output_sender(), |msg| match msg {
                WorkspacesOutput::Clicked(ws) => OverviewWindowOutput::Clicked(ws),
                WorkspacesOutput::ClickedC(id) => OverviewWindowOutput::ClickedC(id),
            });

        let model = Self {
            general: init.general,
            open: false,
            window: root.clone(),
            remove_html: Regex::new(r"<[^>]*>").expect("invalid regex"),
            items,
            #[cfg(feature = "live_windows")]
            live_thumbnails: init.live_thumbnails,
            #[cfg(feature = "live_windows")]
            live_thumbnails_icons: init.live_thumbnails_icons,
            #[cfg(feature = "live_windows")]
            after_paint_clock: None,
            #[cfg(feature = "live_windows")]
            after_paint_handler: Rc::new(RefCell::new(None)),
        };

        let itemsw: gtk::FlowBox = model.items.widget().clone();

        let widgets = view_output!();

        let window = &root;
        window.init_layer_shell();
        window.set_namespace(Some(OVERVIEW_NAMESPACE));
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_monitor(Some(&init.gtk_monitor));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("overview::root::window::update: {message:?}");
        match message {
            OverviewWindowInput::SetGeneral(general) => {
                self.general = general;
            }
            OverviewWindowInput::OpenOverview((data, top_offset)) => {
                if self.open {
                    trace!("already open");
                } else {
                    self.open = true;
                    self.window.set_margin(Edge::Top, i32::from(top_offset));
                    self.open_overview(&data);
                }
            }
            OverviewWindowInput::SetActive(prev, next) => {
                if self.open {
                    self.set_active(prev, next);
                } else {
                    trace!("not open");
                }
            }
            OverviewWindowInput::CloseOverview => {
                if self.open {
                    self.open = false;
                    self.close_overview(&sender);
                } else {
                    trace!("not open");
                }
            }
            OverviewWindowInput::ReloadOverview(data) => {
                if self.open {
                    self.reload_overview(&data, &sender);
                } else {
                    trace!("not open");
                }
            }
            #[cfg(feature = "live_windows")]
            OverviewWindowInput::UpdateClientThumbnail(client_id, texture) => {
                for (idx, _) in self.items.iter().enumerate() {
                    self.items.send(
                        idx,
                        WorkspacesInput::UpdateClientThumbnail(client_id, texture.clone()),
                    );
                }
            }
        }
    }
}

impl OverviewWindow {
    fn open_overview(&mut self, data: &OverviewWindowData) {
        // Cancel any still-pending after-paint hide from a previous close
        // before showing the window again: the old handler must not hide it.
        #[cfg(feature = "live_windows")]
        if let Some(clock) = self.after_paint_clock.take() {
            if let Some(id) = self.after_paint_handler.borrow_mut().take() {
                clock.disconnect(id);
            }
        }

        trace!("Showing window {:?}", self.window.id());
        self.window.set_visible(true);
        self.window.grab_focus();

        self.populate_workspace_mode(data, self.general.scale);
    }

    fn populate_workspace_mode(&mut self, data: &OverviewWindowData, scale: f64) {
        let mut lock = self.items.guard();
        lock.clear();

        for (wid, workspace_data) in &data.workspaces {
            if !workspace_data.any_client_enabled {
                trace!("skipping workspace {} with no enabled clients", wid);
                continue;
            }

            // Get clients for this workspace
            let workspace_clients: Vec<_> = data
                .clients
                .iter()
                .filter(|(_, client)| client.workspace == *wid && client.enabled)
                .map(|(id, data)| (*id, data.clone()))
                .collect();

            lock.push_back(WorkspacesInit {
                monitor_data: data.monitor.clone(),
                remove_html: self.remove_html.clone(),
                id: *wid,
                data: workspace_data.clone(),
                scale,
                clients: workspace_clients,
                #[cfg(feature = "live_windows")]
                live_thumbnails: self.live_thumbnails,
                #[cfg(feature = "live_windows")]
                live_thumbnails_icons: self.live_thumbnails_icons,
            });
        }
        drop(lock);

        // Set active workspace
        for (idx, item) in self.items.iter().enumerate() {
            if item.workspace_id == data.active.workspace {
                if let Some(active_client) = data.active.client {
                    self.items
                        .send(idx, WorkspacesInput::SetActiveClient(active_client));
                } else {
                    self.items.send(idx, WorkspacesInput::SetActive(true));
                }
                break;
            }
        }
    }

    fn set_active(&self, old_active: Active, new_active: Active) {
        for (idx, item) in self.items.iter().enumerate() {
            if item.workspace_id == old_active.workspace
                && item.workspace_id != new_active.workspace
            {
                self.items.send(idx, WorkspacesInput::SetActive(false));
            }
            if item.workspace_id == new_active.workspace {
                if let Some(cid) = new_active.client {
                    self.items.send(idx, WorkspacesInput::SetActiveClient(cid));
                } else if old_active.workspace != new_active.workspace {
                    self.items.send(idx, WorkspacesInput::SetActive(true));
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn close_overview(&mut self, sender: &ComponentSender<Self>) {
        trace!("Hiding window {:?}", self.window.id());
        // Clear UI first so the textures are released before the window is
        // hidden, letting GSK render an empty frame and reclaim the GPU
        // DMA-BUF images.
        {
            let mut lock = self.items.guard();
            lock.clear();
        }

        #[cfg(feature = "live_windows")]
        {
            if let Some(clock) = self.window.frame_clock() {
                // Keep the window visible until one empty frame has been
                // rendered after the textures were cleared, then hide it. The
                // `after-paint` frame clock signal fires once the frame is
                // drawn, so no fixed delay is needed.
                self.after_paint_clock = Some(clock.clone());
                let handler = self.after_paint_handler.clone();
                let window = self.window.downgrade();
                let output = sender.output_sender().clone();
                let id = clock.connect_after_paint(move |clock| {
                    // Disconnect exactly once before hiding so the handler
                    // never runs again, then tell the parent we are hidden.
                    if let Some(id) = handler.borrow_mut().take() {
                        clock.disconnect(id);
                    }
                    if let Some(window) = window.upgrade() {
                        window.set_visible(false);
                    }
                    let _ = output.send(OverviewWindowOutput::Hidden);
                });
                *self.after_paint_handler.borrow_mut() = Some(id);
                self.window.queue_draw();
            } else {
                // No frame clock available: hide directly instead of waiting.
                self.window.set_visible(false);
                let _ = sender.output_sender().send(OverviewWindowOutput::Hidden);
            }
        }
        #[cfg(not(feature = "live_windows"))]
        {
            self.window.set_visible(false);
        }
    }

    fn reload_overview(&mut self, data: &OverviewWindowData, sender: &ComponentSender<Self>) {
        if data.clients.is_empty() {
            self.close_overview(sender);
            return;
        }
        self.populate_workspace_mode(data, self.general.scale);
    }
}

#[derive(Debug)]
pub struct OverviewWindowData {
    pub active: Active,
    pub clients: Vec<(ClientId, ClientData)>,
    pub workspaces: Vec<(WorkspaceId, WorkspaceData)>,
    pub monitor: MonitorData,
}
