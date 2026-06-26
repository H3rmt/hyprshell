use crate::data::{SortConfig, collect_data};
use crate::next::{find_next_client, find_next_workspace};
use crate::overview::window::{
    OverviewWindow, OverviewWindowData, OverviewWindowInit, OverviewWindowInput,
    OverviewWindowOutput,
};
use crate::shared::refresh_captures;
use core_lib::{Active, ByFirst, ClientId, Direction, HyprlandData, MonitorId, WorkspaceId};
use exec_lib::switch::{switch_client, switch_workspace};
use exec_lib::wayland_capture::CaptureManager;
use launcher_lib::{LauncherRoot, LauncherRootInit, LauncherRootInput, LauncherRootOutput};
use relm4::adw::gdk::{Display, Monitor};
use relm4::adw::glib::ControlFlow;
use relm4::adw::prelude::*;
use relm4::adw::{glib, gtk};
use relm4::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use tracing::{debug, error, trace};

const KILL_TIMEOUT: Duration = Duration::from_millis(200);
const THUMBNAIL_BURST_MS: u64 = 8;

#[derive(Debug)]
pub struct OverviewRoot {
    general: config_lib::WindowsGeneral,
    overview: config_lib::Overview,
    open: bool,
    data: OverviewData,

    launcher_root: Controller<LauncherRoot>,
    windows: BTreeMap<MonitorId, Controller<OverviewWindow>>,
    live_thumbnails: bool,
    capture_manager: Option<CaptureManager>,
    timer_handle: Option<glib::SourceId>,
    thumbnail_refresh_ms: u64,
    thumbnail_burst: bool,
}

#[derive(Debug)]
pub enum OverviewRootInput {
    SetOverview(config_lib::Overview),
    SetGeneral(config_lib::WindowsGeneral),
    OpenOverview,
    Switch(Direction, bool),
    CloseOverview(bool),
    CloseOverviewClick(WorkspaceId),
    CloseOverviewClickC(ClientId),
    CloseItem(ClientId),
    ReloadOverview,
    RefreshThumbnails,
}

#[derive(Debug)]
pub struct OverviewRootInit {
    pub general: config_lib::WindowsGeneral,
    pub overview: config_lib::Overview,
    pub data_dir: Rc<PathBuf>,
    pub thumbnail_refresh_ms: u64,
}

#[derive(Debug)]
pub enum OverviewRootOutput {}

#[relm4::component(pub)]
impl SimpleComponent for OverviewRoot {
    type Init = OverviewRootInit;
    type Input = OverviewRootInput;
    type Output = OverviewRootOutput;

    view! {
        gtk::Window {
        }
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        trace!("Initializing OverviewRoot");
        let app = relm4::main_application();
        let mut windows = BTreeMap::new();

        let live_thumbnails = std::env::var_os("HYPRSHELL_EXPERIMENTAL").is_some_and(|v| v == "1");

        let monitors = exec_lib::collect::get_monitors();
        let gmonitors = Display::default()
            .expect("Could not connect to a display")
            .monitors()
            .iter()
            .filter_map(Result::ok)
            .collect::<Vec<Monitor>>();
        for gtk_monitor in gmonitors {
            let monitor_conn = gtk_monitor.connector().unwrap_or_default();
            if let Some(monitor) = monitors.iter().find(|m| m.connector == monitor_conn) {
                let overview_window = OverviewWindow::builder();
                let window = &overview_window.root;
                app.add_window(window);
                let overview_window = overview_window
                    .launch(OverviewWindowInit {
                        general: init.general.clone(),
                        gtk_monitor,
                        live_thumbnails,
                    })
                    .forward(sender.input_sender(), |m| match m {
                        OverviewWindowOutput::Clicked(ws) => {
                            OverviewRootInput::CloseOverviewClick(ws)
                        }
                        OverviewWindowOutput::ClickedC(cl) => {
                            OverviewRootInput::CloseOverviewClickC(cl)
                        }
                    });
                windows.entry(monitor.id).insert_entry(overview_window);
            }
        }
        let launcher_root = LauncherRoot::builder();
        let window = &launcher_root.root;
        app.add_window(window);
        let launcher_root = launcher_root
            .launch(LauncherRootInit {
                launcher: init.overview.launcher.clone(),
                data_dir: init.data_dir,
            })
            .forward(sender.input_sender(), |msg| match msg {
                LauncherRootOutput::Switch(dir, ws) => OverviewRootInput::Switch(dir, ws),
                LauncherRootOutput::Close(do_switch) => OverviewRootInput::CloseOverview(do_switch),
            });

        let model = Self {
            general: init.general,
            overview: init.overview,
            open: false,
            windows,
            launcher_root,
            data: OverviewData::default(),
            live_thumbnails,
            capture_manager: None,
            timer_handle: None,
            thumbnail_refresh_ms: init.thumbnail_refresh_ms,
            thumbnail_burst: false,
        };

        let widgets = view_output!();
        sender
            .input_sender()
            .emit(OverviewRootInput::SetOverview(model.overview.clone()));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("overview::root::update: {message:?}");
        match message {
            OverviewRootInput::SetOverview(overview) => {
                self.overview = overview;
                self.launcher_root.emit(LauncherRootInput::SetLauncher(
                    self.overview.launcher.clone(),
                ));
            }
            OverviewRootInput::SetGeneral(general) => {
                for window in self.windows.values_mut() {
                    window.emit(OverviewWindowInput::SetGeneral(general.clone()));
                }
                self.general = general;
            }
            OverviewRootInput::OpenOverview => {
                if self.open {
                    sender
                        .input_sender()
                        .emit(OverviewRootInput::CloseOverview(false));
                } else {
                    self.open = true;
                    self.launcher_root.emit(LauncherRootInput::OpenLauncher);
                    self.open_overview(&sender);
                }
            }
            OverviewRootInput::Switch(direction, workspace) => {
                if self.open {
                    self.navigate(direction, workspace);
                } else {
                    trace!("not open");
                }
            }
            OverviewRootInput::CloseOverview(do_switch) => {
                if self.open {
                    self.open = false;
                    self.launcher_root.emit(LauncherRootInput::CloseLauncher);
                    self.close_overview(do_switch);
                } else {
                    trace!("not open");
                }
            }
            OverviewRootInput::CloseItem(id) => {
                if self.open {
                    self.close_item(id);
                } else {
                    trace!("not open");
                }
                sender
                    .input_sender()
                    .emit(OverviewRootInput::ReloadOverview);
            }
            OverviewRootInput::ReloadOverview => {
                if self.open {
                    self.reload_overview();
                } else {
                    trace!("not open");
                }
            }
            OverviewRootInput::CloseOverviewClick(ws) => {
                self.data.active.client = None;
                self.data.active.workspace = ws;
                sender
                    .input_sender()
                    .emit(OverviewRootInput::CloseOverview(true));
            }
            OverviewRootInput::CloseOverviewClickC(cl) => {
                self.data.active.client = Some(cl);
                sender
                    .input_sender()
                    .emit(OverviewRootInput::CloseOverview(true));
            }
            OverviewRootInput::RefreshThumbnails => self.refresh_thumbnails(&sender),
        }
    }
}

impl OverviewRoot {
    fn open_overview(&mut self, sender: &ComponentSender<Self>) {
        let (hypr_data, active) = match collect_data(&SortConfig {
            filter_current_monitor: self.overview.filter_by_current_monitor,
            filter_current_workspace: self.overview.filter_by_current_workspace,
            filter_same_class: self.overview.filter_by_same_class,
            sort_recent: false,
            exclude_workspaces: if self.overview.exclude_workspaces.is_empty() {
                None
            } else {
                Some(self.overview.exclude_workspaces.clone())
            },
        }) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to collect data: {}", e);
                return;
            }
        };
        self.data = OverviewData {
            active,
            hypr_data: hypr_data.clone(),
        };
        self.render(hypr_data, self.data.active, true);

        if self.live_thumbnails {
            self.capture_manager = CaptureManager::new().map_err(|e| error!("{e}")).ok();
            self.thumbnail_burst = true;
            let sender = sender.clone();
            self.timer_handle = Some(glib::timeout_add_local(
                Duration::from_millis(THUMBNAIL_BURST_MS),
                move || {
                    sender.input(OverviewRootInput::RefreshThumbnails);
                    ControlFlow::Continue
                },
            ));
        }
    }

    fn navigate(&mut self, direction: Direction, workspace: bool) {
        let new_active = if workspace {
            find_next_workspace(
                direction,
                false,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        } else {
            if direction == Direction::Up || direction == Direction::Down {
                error!(
                    "Clients in overview can only be switched left and right (forwards and backwards)"
                );
                return;
            }
            find_next_client(
                direction,
                false,
                &self.data.hypr_data,
                self.data.active,
                self.general.items_per_row,
            )
        };

        let old_active = self.data.active;
        self.data.active = new_active;

        if new_active != old_active {
            for window in self.windows.values() {
                window.emit(OverviewWindowInput::SetActive(old_active, new_active));
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn close_item(&self, id: ClientId) {
        if let Err(e) = exec_lib::kill::kill_client_blocking(id, KILL_TIMEOUT) {
            // TODO: close on killed to let user close window themself
            tracing::warn!("Failed to kill client {id}: {e}");
        }
    }

    fn close_overview(&mut self, do_switch: bool) {
        for window in self.windows.values() {
            window.emit(OverviewWindowInput::CloseOverview);
        }

        if do_switch {
            if let Some(id) = self.data.active.client {
                debug!(
                    "Switching to client {}",
                    self.data
                        .hypr_data
                        .clients
                        .iter()
                        .find(|(cid, _)| *cid == id)
                        .map_or_else(|| "<Unknown>".to_string(), |(_, c)| c.title.clone())
                );
                // Defer execution to ensure window is hidden first
                glib::idle_add_local(move || {
                    if let Err(e) = switch_client(id) {
                        tracing::warn!("Failed to switch to client {id:?}: {e}");
                    }
                    ControlFlow::Break
                });
            } else {
                let id = self.data.active.workspace;
                debug!(
                    "Switching to workspace {}",
                    self.data
                        .hypr_data
                        .workspaces
                        .iter()
                        .find(|(wid, _)| *wid == id)
                        .map_or_else(|| "<Unknown>".to_string(), |(_, w)| w.name.clone())
                );
                glib::idle_add_local(move || {
                    if let Err(e) = switch_workspace(id) {
                        tracing::warn!("Failed to switch to workspace {id:?}: {e}");
                    }
                    ControlFlow::Break
                });
            }
        }

        if let Some(handle) = self.timer_handle.take() {
            handle.remove();
        }
        self.capture_manager = None;
    }

    fn reload_overview(&mut self) {
        let (hypr_data, _active) = match collect_data(&SortConfig {
            filter_current_monitor: self.overview.filter_by_current_monitor,
            filter_current_workspace: self.overview.filter_by_current_workspace,
            filter_same_class: self.overview.filter_by_same_class,
            sort_recent: false,
            exclude_workspaces: if self.overview.exclude_workspaces.is_empty() {
                None
            } else {
                Some(self.overview.exclude_workspaces.clone())
            },
        }) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to collect data: {}", e);
                return;
            }
        };

        while match self.data.active {
            Active {
                client: Some(id), ..
            } => hypr_data.clients.find_by_first(&id).is_none(),
            Active { workspace: id, .. } => hypr_data.workspaces.find_by_first(&id).is_none(),
        } {
            self.data.active = find_next_workspace(
                Direction::Right,
                true,
                &hypr_data,
                self.data.active,
                self.general.items_per_row,
            );
        }

        self.data = OverviewData {
            active: self.data.active,
            hypr_data: hypr_data.clone(),
        };
        self.render(hypr_data, self.data.active, false);
    }

    fn render(&self, hypr_data: HyprlandData, active: Active, open: bool) {
        let mut mapped_ws = BTreeMap::new();
        for (i, workspace_data) in hypr_data.workspaces {
            mapped_ws
                .entry(workspace_data.monitor)
                .or_insert_with(Vec::new)
                .push((i, workspace_data));
        }
        let mut mapped_cl = BTreeMap::new();
        for (i, client_data) in hypr_data.clients {
            mapped_cl
                .entry(client_data.monitor)
                .or_insert_with(Vec::new)
                .push((i, client_data));
        }

        for (monitor_id, window) in &self.windows {
            if let Some(data) = hypr_data.monitors.find_by_first(monitor_id) {
                // always update, maybe last client got removed
                let data = OverviewWindowData {
                    active,
                    clients: mapped_cl.remove(monitor_id).unwrap_or_default(),
                    workspaces: mapped_ws.remove(monitor_id).unwrap_or_default(),
                    monitor: data.clone(),
                };
                if open {
                    window.emit(OverviewWindowInput::OpenOverview((
                        data,
                        self.overview.top_offset,
                    )));
                } else {
                    window.emit(OverviewWindowInput::ReloadOverview(data));
                }
            }
        }
    }

    fn refresh_thumbnails(&mut self, sender: &ComponentSender<Self>) {
        let Some(mgr) = &mut self.capture_manager else {
            return;
        };
        let Some(display) = Display::default() else {
            return;
        };
        let captures = refresh_captures(mgr, &display, !self.thumbnail_burst);
        if self.thumbnail_burst && mgr.pending_count() == 0 {
            self.thumbnail_burst = false;
            // all initial thumbnails are loaded
            // remove initial thumbnail burst timer
            if let Some(h) = self.timer_handle.take() {
                h.remove();
            }
            // start new slower timer if thumbnail_refresh_ms is set
            if self.thumbnail_refresh_ms != 0 {
                trace!("Switching from thumbnail_burst refresh to slow refresh");
                let sender = sender.clone();
                self.timer_handle = Some(glib::timeout_add_local(
                    Duration::from_millis(self.thumbnail_refresh_ms),
                    move || {
                        sender.input(OverviewRootInput::RefreshThumbnails);
                        ControlFlow::Continue
                    },
                ));
            } else {
                trace!("All initial thumbnail captures loaded");
            }
        }

        for (client_id, texture) in captures {
            for window in self.windows.values() {
                window.emit(OverviewWindowInput::UpdateClientThumbnail(
                    client_id,
                    texture.clone(),
                ));
            }
        }
    }
}

#[derive(Debug)]
pub struct OverviewData {
    pub active: Active,
    pub hypr_data: HyprlandData,
}

impl Default for OverviewData {
    fn default() -> Self {
        Self {
            active: Active {
                client: None,
                workspace: -1,
                monitor: -1,
            },
            hypr_data: HyprlandData::default(),
        }
    }
}
