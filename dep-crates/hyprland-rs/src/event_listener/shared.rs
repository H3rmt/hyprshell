use crate::shared::*;
use std::{fmt::Debug, pin::Pin};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActiveWindowValue<T> {
    Queued(T), // aka Some(T)
    None,      // No current window
    Empty,     // Empty queue
}

impl<T> ActiveWindowValue<T> {
    pub fn reset(&mut self) {
        *self = Self::Empty;
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveWindowState {
    pub class: ActiveWindowValue<String>,
    pub title: ActiveWindowValue<String>,
    pub addr: ActiveWindowValue<Address>,
}

pub(crate) trait HasExecutor {
    fn event_executor(&mut self, event: Event) -> crate::Result<()>;

    fn event_primer(&mut self, event: Event, abuf: &mut Vec<ActiveWindowState>) -> crate::Result<()>
    where
        Self: Sized,
    {
        if abuf.is_empty() {
            abuf.push(ActiveWindowState::new());
        }
        if let Event::ActiveWindowChangedV1(data) = event {
            let mut to_remove = vec![];
            let data = into(data);
            for (index, awin) in abuf.iter_mut().enumerate() {
                if awin.title.is_empty() && awin.class.is_empty() {
                    (awin.class, awin.title) = data.clone();
                }
                if awin.ready() {
                    awin.execute(self)?;
                    to_remove.push(index);
                    break;
                }
            }
            for index in to_remove.into_iter().rev() {
                abuf.swap_remove(index);
            }
        } else if let Event::ActiveWindowChangedV2(data) = event {
            let mut to_remove = vec![];
            for (index, awin) in abuf.iter_mut().enumerate() {
                if awin.addr.is_empty() {
                    awin.addr = data.clone().into();
                }
                if awin.ready() {
                    awin.execute(self)?;
                    to_remove.push(index);
                    break;
                }
            }
            for index in to_remove.into_iter().rev() {
                abuf.swap_remove(index);
            }
        } else {
            self.event_executor(event)?;
        }
        Ok(())
    }
}

const BLACKLISTED_UNKNOWN_EVENTS: &[&str; 10] = &[
    "workspace",
    "focusedmon",
    "monitorremoved",
    "monitoradded",
    "createworkspace",
    "destroyworkspace",
    "moveworkspace",
    "activespecial",
    "movewindow",
    "windowtitle",
];

#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub(crate) fn event_primer_noexec(
    event: Event,
    abuf: &mut Vec<ActiveWindowState>,
) -> crate::Result<Vec<Event>> {
    if abuf.is_empty() {
        abuf.push(ActiveWindowState::new());
    }
    let mut events: Vec<Event> = vec![];
    if let Event::ActiveWindowChangedV1(data) = event {
        let mut to_remove = vec![];
        let data = into(data);
        for (index, awin) in abuf.iter_mut().enumerate() {
            if awin.title.is_empty() && awin.class.is_empty() {
                (awin.class, awin.title) = data.clone();
            }
            if awin.ready() {
                if let Some(event) = awin.get_event() {
                    events.push(event);
                };
                to_remove.push(index);
                break;
            }
        }
        for index in to_remove.into_iter().rev() {
            abuf.swap_remove(index);
        }
    } else if let Event::ActiveWindowChangedV2(data) = event {
        let mut to_remove = vec![];
        for (index, awin) in abuf.iter_mut().enumerate() {
            if awin.addr.is_empty() {
                awin.addr = data.clone().into();
            }
            if awin.ready() {
                if let Some(event) = awin.get_event() {
                    events.push(event);
                };
                to_remove.push(index);
                break;
            }
        }
        for index in to_remove.into_iter().rev() {
            abuf.swap_remove(index);
        }
    } else if let Event::Unknown(ref data) = event {
        if !BLACKLISTED_UNKNOWN_EVENTS.contains(&&*data.name) {
            events.push(event);
        }
    } else {
        events.push(event);
    }
    Ok(events)
}

#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub(crate) trait HasAsyncExecutor {
    async fn event_executor_async(&mut self, event: Event) -> crate::Result<()>;

    async fn event_primer_exec_async(
        &mut self,
        event: Event,
        abuf: &mut Vec<ActiveWindowState>,
    ) -> crate::Result<()>
    where
        Self: Sized,
    {
        for x in event_primer_noexec(event, abuf)? {
            self.event_executor_async(x).await?;
        }
        Ok(())
    }
}

impl ActiveWindowState {
    pub fn execute<T: HasExecutor>(&mut self, listener: &mut T) -> crate::Result<()> {
        use ActiveWindowValue::{None, Queued};
        let data = (&self.title, &self.class, &self.addr);
        if let (Queued(title), Queued(class), Queued(addr)) = data {
            listener.event_executor(Event::ActiveWindowChanged(Some(WindowEventData {
                class: class.to_string(),
                title: title.to_string(),
                address: addr.clone(),
            })))?;
            self.reset();
        } else if let (None, None, None) = data {
            listener.event_executor(Event::ActiveWindowChanged(Option::None))?;
        }
        Ok(())
    }
    pub fn get_event(&mut self) -> Option<Event> {
        use ActiveWindowValue::{None, Queued};
        let data = (&self.title, &self.class, &self.addr);
        let mut event = Option::None;
        if let (Queued(title), Queued(class), Queued(addr)) = data {
            event = Some(Event::ActiveWindowChanged(Some(WindowEventData {
                class: class.to_string(),
                title: title.to_string(),
                address: addr.clone(),
            })));
            self.reset();
        } else if let (None, None, None) = data {
            event = Some(Event::ActiveWindowChanged(Option::None));
        }
        event
    }

    pub fn ready(&self) -> bool {
        !self.class.is_empty() && !self.title.is_empty() && !self.addr.is_empty()
    }
    pub fn reset(&mut self) {
        self.class.reset();
        self.title.reset();
        self.addr.reset();
    }
    pub fn new() -> Self {
        Self {
            class: ActiveWindowValue::Empty,
            title: ActiveWindowValue::Empty,
            addr: ActiveWindowValue::Empty,
        }
    }
}

impl<T> From<Option<T>> for ActiveWindowValue<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => ActiveWindowValue::Queued(v),
            None => ActiveWindowValue::None,
        }
    }
}

pub(crate) fn into<T>(from: Option<(T, T)>) -> (ActiveWindowValue<T>, ActiveWindowValue<T>) {
    if let Some((first, second)) = from {
        (
            ActiveWindowValue::Queued(first),
            ActiveWindowValue::Queued(second),
        )
    } else {
        (ActiveWindowValue::None, ActiveWindowValue::None)
    }
}

pub(crate) type EventType<T> = Box<T>;
pub(crate) type AsyncEventType<T> = Pin<Box<T>>;

pub(crate) type VoidFuture = Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

pub(crate) type EmptyClosure = EventType<dyn Fn()>;
pub(crate) type Closure<T> = EventType<dyn Fn(T)>;
pub(crate) type AsyncClosure<T> = AsyncEventType<dyn Sync + Send + Fn(T) -> VoidFuture>;
pub(crate) type EmptyAsyncClosure = AsyncEventType<dyn Sync + Send + Fn() -> VoidFuture>;
pub(crate) type Closures<T> = Vec<Closure<T>>;
pub(crate) type AsyncClosures<T> = Vec<AsyncClosure<T>>;

/// Event data for screencast event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreencastEventData {
    /// State/Is it turning on?
    pub turning_on: bool,
    /// Owner type, is it a monitor?
    pub monitor: bool,
}

/// The data for the event executed when moving a window to a new workspace
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowMoveEvent {
    /// Window address
    pub window_address: Address,
    /// the workspace id
    pub workspace_id: WorkspaceId,
    /// The workspace name
    pub workspace_name: WorkspaceType,
}

/// The data for the event executed when opening a new window
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowOpenEvent {
    /// Window address
    pub window_address: Address,
    /// The workspace name
    pub workspace_name: String,
    /// Window class
    pub window_class: String,
    /// Window title
    pub window_title: String,
}

/// The data for the event executed when changing keyboard layouts
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutEvent {
    /// Keyboard name
    pub keyboard_name: String,
    /// Layout name
    pub layout_name: String,
}

/// The mutable state available to Closures
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct State {
    /// The active workspace
    pub active_workspace: WorkspaceType,
    /// The active monitor
    pub active_monitor: String,
    /// The fullscreen state
    pub fullscreen_state: bool,
}

impl State {
    /// Execute changes in state
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn execute_state(self, old: State) -> crate::Result<Self> {
        self.instance_execute_state(default_instance()?, old).await
    }

    /// Execute changes in state
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_execute_state(
        self,
        instance: &Instance,
        old: State,
    ) -> crate::Result<Self> {
        let state = self.clone();
        if self != old {
            use crate::dispatch::{Dispatch, DispatchType};
            if old.fullscreen_state != state.fullscreen_state {
                use crate::dispatch::FullscreenType;
                Dispatch::instance_call_async(
                    instance,
                    DispatchType::ToggleFullscreen(FullscreenType::NoParam),
                )
                .await?;
            }
            if old.active_workspace != state.active_workspace {
                use crate::dispatch::WorkspaceIdentifierWithSpecial;
                Dispatch::instance_call_async(
                    instance,
                    DispatchType::Workspace(match &state.active_workspace {
                        WorkspaceType::Regular(name) => WorkspaceIdentifierWithSpecial::Name(name),
                        WorkspaceType::Special(opt) => {
                            WorkspaceIdentifierWithSpecial::Special(match opt {
                                Some(name) => Some(name),
                                None => None,
                            })
                        }
                    }),
                )
                .await?;
            }
            if old.active_monitor != state.active_monitor {
                use crate::dispatch::MonitorIdentifier;
                Dispatch::instance_call_async(
                    instance,
                    DispatchType::FocusMonitor(MonitorIdentifier::Name(&state.active_monitor)),
                )
                .await?;
            };
        }
        Ok(state)
    }

    /// Execute changes in state
    pub fn execute_state_sync(self, old: State) -> crate::Result<Self> {
        self.instance_execute_state_sync(default_instance()?, old)
    }

    /// Execute changes in state
    pub fn instance_execute_state_sync(
        self,
        instance: &Instance,
        old: State,
    ) -> crate::Result<Self> {
        let state = self.clone();
        if self != old {
            use crate::dispatch::{Dispatch, DispatchType};
            if old.fullscreen_state != state.fullscreen_state {
                use crate::dispatch::FullscreenType;
                Dispatch::instance_call(
                    instance,
                    DispatchType::ToggleFullscreen(FullscreenType::NoParam),
                )?;
            }
            if old.active_workspace != state.active_workspace {
                use crate::dispatch::WorkspaceIdentifierWithSpecial;
                Dispatch::instance_call(
                    instance,
                    DispatchType::Workspace(match &state.active_workspace {
                        WorkspaceType::Regular(name) => WorkspaceIdentifierWithSpecial::Name(name),
                        WorkspaceType::Special(opt) => {
                            WorkspaceIdentifierWithSpecial::Special(match opt {
                                Some(name) => Some(name),
                                None => None,
                            })
                        }
                    }),
                )?;
            }
            if old.active_monitor != state.active_monitor {
                use crate::dispatch::MonitorIdentifier;
                Dispatch::instance_call(
                    instance,
                    DispatchType::FocusMonitor(MonitorIdentifier::Name(&state.active_monitor)),
                )?;
            };
        }
        Ok(state)
    }
}

pub(crate) fn execute_empty_closure(f: &EmptyClosure) {
    f();
}

pub(crate) fn execute_closure<T: Clone>(f: &Closure<T>, val: T) {
    f(val);
}

#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub(crate) async fn execute_empty_closure_async(f: &EmptyAsyncClosure) {
    f().await;
}

#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub(crate) async fn execute_closure_async<T>(f: &AsyncClosure<T>, val: T) {
    f(val).await;
}

/// This struct holds workspace event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceEventData {
    /// The workspace name
    pub name: WorkspaceType,
    /// The window id
    pub id: WorkspaceId,
}

/// This struct holds workspace event data
/// when the workspace cannot be special
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonSpecialWorkspaceEventData {
    /// The workspace name
    pub name: String,
    /// The window id
    pub id: WorkspaceId,
}

/// This struct holds workspace moved event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMovedEventData {
    /// The workspace name
    pub name: WorkspaceType,
    /// The window id
    pub id: WorkspaceId,
    /// The monitor name
    pub monitor: String,
}

/// This struct holds window event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowEventData {
    /// The window class
    pub class: String,
    /// The window title
    pub title: String,
    /// The window address
    pub address: Address,
}

/// This struct holds monitor event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorEventData {
    /// The monitor name
    pub monitor_name: String,
    /// The workspace name
    pub workspace_name: Option<WorkspaceType>,
}

/// This struct holds changed special event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedSpecialEventData {
    /// The monitor name
    pub monitor_name: String,
    /// The workspace name
    pub workspace_name: String,
}

/// This struct holds monitor event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorAddedEventData {
    /// The monitor's id
    pub id: MonitorId,
    /// The monitor's name
    pub name: String,
    /// the monitor's description
    pub description: String,
}

/// This struct holds window float event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFloatEventData {
    /// The window address
    pub address: Address,
    /// The float state
    pub floating: bool,
}

/// This struct holds window pin event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowPinEventData {
    /// The window address
    pub address: Address,
    /// The pin state
    pub pinned: bool,
}

/// This struct holds the event data for the windowtitle changed event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowTitleEventData {
    /// The window address
    pub address: Address,
    /// The window title
    pub title: String,
}

/// This struct represents an unknown event to hyprland-rs
/// this allows you to use events that haven't been implemented in hyprland-rs.
/// To use this use the [UnknownEventData::parse_args] method to properly get the args
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEventData {
    /// The event's name
    pub name: String,
    /// The args as a string
    pub args: String,
}

impl UnknownEventData {
    /// Takes the amount of args, and splits the string correctly
    pub fn parse_args(self, count: usize) -> Vec<String> {
        self.args
            .splitn(count, ",")
            .map(|x| x.to_string())
            .collect()
    }
}
/// This struct holds the data for the [Event::GroupToggled] event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupToggledEventData {
    /// The toggle status, `false` means the group was destroyed
    pub toggled: bool,
    /// The window addresses associated with the group
    pub window_addresses: Vec<Address>,
}

/// This enum holds every event type
#[derive(Debug, Clone)]
pub enum Event {
    /// An unknown event
    Unknown(UnknownEventData),
    /// An event that emits when the current workspace is changed,
    /// it is the equivelant of the `workspacev2` event
    WorkspaceChanged(WorkspaceEventData),
    /// An event that emits when a workspace is deleted,
    /// it is the equivelant of the `destroyworkspacev2` event
    WorkspaceDeleted(WorkspaceEventData),
    /// An event that emits when a workspace is created,
    /// it is the equivelant of the `createworkspacev2` event
    WorkspaceAdded(WorkspaceEventData),
    /// An event that emits when a workspace is moved to another monitor,
    /// it is the equivelant of the `moveworkspacev2` event
    WorkspaceMoved(WorkspaceMovedEventData),
    /// An event that emits when a workspace is renamed,
    /// it is the equivelant of the `renameworkspace` event
    WorkspaceRenamed(NonSpecialWorkspaceEventData),
    #[doc(hidden)]
    ActiveWindowChangedV1(Option<(String, String)>), // internal intermediary event
    #[doc(hidden)]
    ActiveWindowChangedV2(Option<Address>), // internal intermediary event
    /// An event that emits when the active window is changed
    /// Unlike the other events, this is a combination of 2 events
    /// Those being `activewindow` and `activewindowv2`,
    /// it waits for both, and then sends one unified event :)
    ActiveWindowChanged(Option<WindowEventData>),
    /// An event that emits when the active monitor is changed,
    /// it is the equivelant of the `focusedmon` event
    ActiveMonitorChanged(MonitorEventData),
    /// An event that emits when the current fullscreen state is changed,
    /// it is the equivelant of the `fullscreen` event
    FullscreenStateChanged(bool),
    /// An event that emits when a new monitor is added/connected,
    /// it is the equivelant of the `monitoraddedv2` event
    MonitorAdded(MonitorAddedEventData),
    /// An event that emits when a monitor is removed/disconnected,
    /// it is the equivelant of the `monitorremoved` event
    MonitorRemoved(String),
    /// An event that emits when a window is opened,
    /// it is the equivelant of the `openwindow` event
    WindowOpened(WindowOpenEvent),
    /// An event that emits when a window is closed,
    /// it is the equivelant of the `closewindow` event
    WindowClosed(Address),
    /// An event that emits when a window is moved to a different workspace,
    /// it is the equivelant of the `movewindowv2` event
    WindowMoved(WindowMoveEvent),
    /// An event that emits when a special workspace is closed on the current monitor,
    /// it is the equivelant of the `activespecial` event
    SpecialRemoved(String),
    /// An event that emits when the current special workspace is changed on a monitor,
    /// it is the equivelant of the `activespecial` event
    ChangedSpecial(ChangedSpecialEventData),
    /// An event that emits when the layout of a keyboard changes,
    /// it is the equivelant of the `activelayout` event
    LayoutChanged(LayoutEvent),
    /// An event that emits when the current keybind submap changes,
    /// it is the equivelant of the `submap` event
    SubMapChanged(String),
    /// An event that emits when a layer shell surface is opened/mapped,
    /// it is the equivelant of the `openlayer` event
    LayerOpened(String),
    /// An event that emits when a layer shell surface is closed/unmapped,
    /// it is the equivelant of the `closelayer` event
    LayerClosed(String),
    /// An event that emits when the floating state of a window changes,
    /// it is the equivelant of the `changefloatingmode` event
    FloatStateChanged(WindowFloatEventData),
    /// An event that emits when the a window requests the urgent state,
    /// it is the equivelant of the `urgent` event
    UrgentStateChanged(Address),
    /// An event that emits when the title of a window changes,
    /// it is the equivelant of the `windowtitlev2` event
    WindowTitleChanged(WindowTitleEventData),
    /// An event that emits when the screencopy state of a client changes
    /// AKA, a process wants to capture/record your screen,
    /// it is the equivelant of the `screencast` event
    Screencast(ScreencastEventData),
    /// An event that emits when hyprland is reloaded,
    /// it is the equivelant of the `configreloaded` event
    ConfigReloaded,
    /// An event that emits when `ignoregrouplock` is toggled,
    /// it is the equivelant of the `ignoregrouplock` event
    IgnoreGroupLockStateChanged(bool),
    /// An event that emits when `lockgroups` is toggled,
    /// it is the equivelant of the `lockgroups` event
    LockGroupsStateChanged(bool),
    /// An event that emits when a window is pinned or unpinned,
    /// it is the equivelant of the `pin` event
    WindowPinned(WindowPinEventData),
    /// And event that emits when a group is toggled,
    /// it is the equivelant of the `togglegroup`
    GroupToggled(GroupToggledEventData),
    /// And event that emits when a window is moved into a group,
    /// it is the equivelant of the `moveintogroup`
    WindowMovedIntoGroup(Address),
    /// And event that emits when a window is moved out of a group,
    /// it is the equivelant of the `moveoutofgroup`
    WindowMovedOutOfGroup(Address),
}

fn parse_string_as_work(str: String) -> WorkspaceType {
    if str == "special" {
        WorkspaceType::Special(None)
    } else if str.starts_with("special:") {
        {
            let mut iter = str.split(':');
            iter.next();
            match iter.next() {
                Some(name) => WorkspaceType::Special(Some(name.to_string())),
                None => WorkspaceType::Special(None),
            }
        }
    } else {
        WorkspaceType::Regular(str)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub(crate) enum ParsedEventType {
    WorkspaceChangedV2,
    WorkspaceDeletedV2,
    WorkspaceAddedV2,
    WorkspaceMovedV2,
    WorkspaceRename,
    ActiveWindowChangedV1,
    ActiveWindowChangedV2,
    ActiveMonitorChanged,
    FullscreenStateChanged,
    MonitorAddedV2,
    MonitorRemoved,
    WindowOpened,
    WindowClosed,
    WindowMovedV2,
    ActiveSpecial,
    LayoutChanged,
    SubMapChanged,
    LayerOpened,
    LayerClosed,
    FloatStateChanged,
    UrgentStateChanged,
    WindowTitleChangedV2,
    Screencast,
    ConfigReloaded,
    IgnoreGroupLock,
    LockGroups,
    Pin,
    ToggleGroup,
    MoveIntoGroup,
    MoveOutOfGroup,
}

/// All Hyprland events's arg count and enum variant.
/// The first item of the tuple is a usize of the argument count
/// This allows for easy parsing because the last arg in a Hyprland event
/// has the ability to have extra `,`s
pub(crate) static EVENTS: &[(&str, (usize, ParsedEventType))] = &[
    ("workspacev2", (2, ParsedEventType::WorkspaceChangedV2)),
    (
        "destroyworkspacev2",
        (2, ParsedEventType::WorkspaceDeletedV2),
    ),
    ("createworkspacev2", (2, ParsedEventType::WorkspaceAddedV2)),
    ("moveworkspacev2", (3, ParsedEventType::WorkspaceMovedV2)),
    ("renameworkspace", (2, ParsedEventType::WorkspaceRename)),
    ("focusedmon", (2, ParsedEventType::ActiveMonitorChanged)),
    ("activewindow", (2, ParsedEventType::ActiveWindowChangedV1)),
    (
        "activewindowv2",
        (1, ParsedEventType::ActiveWindowChangedV2),
    ),
    ("fullscreen", (1, ParsedEventType::FullscreenStateChanged)),
    ("monitorremoved", (1, ParsedEventType::MonitorRemoved)),
    ("monitoraddedv2", (3, ParsedEventType::MonitorAddedV2)),
    ("openwindow", (4, ParsedEventType::WindowOpened)),
    ("closewindow", (1, ParsedEventType::WindowClosed)),
    ("movewindowv2", (3, ParsedEventType::WindowMovedV2)),
    ("activelayout", (2, ParsedEventType::LayoutChanged)),
    ("activespecial", (2, ParsedEventType::ActiveSpecial)),
    ("submap", (1, ParsedEventType::SubMapChanged)),
    ("openlayer", (1, ParsedEventType::LayerOpened)),
    ("closelayer", (1, ParsedEventType::LayerClosed)),
    (
        "changefloatingmode",
        (2, ParsedEventType::FloatStateChanged),
    ),
    ("screencast", (2, ParsedEventType::Screencast)),
    ("urgent", (1, ParsedEventType::UrgentStateChanged)),
    ("windowtitlev2", (2, ParsedEventType::WindowTitleChangedV2)),
    ("configreloaded", (0, ParsedEventType::ConfigReloaded)),
    ("ignoregrouplock", (1, ParsedEventType::IgnoreGroupLock)),
    ("lockgroups", (1, ParsedEventType::LockGroups)),
    ("pin", (2, ParsedEventType::Pin)),
    ("togglegroup", (2, ParsedEventType::ToggleGroup)),
    ("moveintogroup", (1, ParsedEventType::MoveIntoGroup)),
    ("moveoutofgroup", (1, ParsedEventType::MoveOutOfGroup)),
];

use crate::default_instance;
use crate::error::HyprError;
use crate::instance::Instance;
use either::Either;

type KnownEvent = (ParsedEventType, Vec<String>);
type UnknownEvent = (String, String);

fn new_event_parser(input: &str) -> crate::Result<Either<KnownEvent, UnknownEvent>> {
    input
        .to_string()
        .split_once(">>")
        .ok_or(HyprError::Other(
            "could not get event name from Hyprland IPC data (not hyprland-rs)".to_string(),
        ))
        .map(|(name, x)| {
            if let Some(event) = EVENTS
                .iter()
                .find(|(i_name, _)| *i_name == name)
                .map(|(_, event)| event)
            {
                Either::Left((
                    event.1,
                    x.splitn(event.0, ",").map(|y| y.to_string()).collect(),
                ))
            } else {
                Either::Right((name.to_string(), x.to_string()))
            }
        })
}

macro_rules! parse_int {
    ($int:expr, event: $event:literal) => {
        parse_int!($int, event: $event => WorkspaceId)
    };
    ($int:expr, event: $event:literal => $int_type:ty) => {
        ($int
            .parse::<$int_type>()
            .map_err(|e|
                HyprError::Internal(format!(concat!($event, ": invalid integer error: {}"), e))
             )?
        )
    };

}

macro_rules! get {
    ($args:expr ; $id:literal) => {
        get![ref $args;$id].clone()
    };
    (ref $args:expr ; $id:literal) => {
        $args
            .get($id)
            .ok_or(HyprError::Internal(
                concat!("could not get the event arg of index ", stringify!($id)).to_string(),
            ))?
    };
}

/// This internal function parses event strings
pub(crate) fn event_parser(event: &str) -> crate::Result<Vec<Event>> {
    // TODO: Optimize nested looped regex capturing. Maybe pull in rayon if possible.
    let event_iter = event.trim().lines().filter_map(|event_line| {
        if event_line.is_empty() {
            None
        } else {
            Some(new_event_parser(event_line))
        }
    });

    let parsed_events = event_iter.map(|event| match event {
        Err(x) => Err(x),
        Ok(Either::Right((name, args))) => Ok(Event::Unknown(UnknownEventData { name, args })),
        Ok(Either::Left((event_type, args))) => match event_type {
            ParsedEventType::WorkspaceChangedV2 => {
                Ok(Event::WorkspaceChanged(WorkspaceEventData {
                    id: parse_int!(get![ref args;0], event: "WorkspaceChangedV2"),
                    name: parse_string_as_work(get![args;1]),
                }))
            }
            ParsedEventType::WorkspaceDeletedV2 => {
                Ok(Event::WorkspaceDeleted(WorkspaceEventData {
                    id: parse_int!(get![ref args;0], event: "WorkspaceDeletedV2"),
                    name: parse_string_as_work(get![args;1]),
                }))
            }
            ParsedEventType::WorkspaceAddedV2 => Ok(Event::WorkspaceAdded(WorkspaceEventData {
                id: parse_int!(get![ref args;0], event: "WorkspaceAddedV2"),
                name: parse_string_as_work(get![args;1]),
            })),
            ParsedEventType::WorkspaceMovedV2 => {
                Ok(Event::WorkspaceMoved(WorkspaceMovedEventData {
                    id: parse_int!(get![ref args;0], event: "WorkspaceMovedV2"),
                    name: parse_string_as_work(get![args;1]),
                    monitor: get![args;2],
                }))
            }
            ParsedEventType::WorkspaceRename => {
                Ok(Event::WorkspaceRenamed(NonSpecialWorkspaceEventData {
                    id: parse_int!(get![args;0], event: "WorkspaceRenamed"),
                    name: get![args;1],
                }))
            }
            ParsedEventType::ActiveMonitorChanged => {
                Ok(Event::ActiveMonitorChanged(MonitorEventData {
                    monitor_name: get![args;0],
                    workspace_name: if get![args;1] == "?" {
                        None
                    } else {
                        Some(parse_string_as_work(get![args;1]))
                    },
                }))
            }
            ParsedEventType::ActiveWindowChangedV1 => {
                let class = get![args;0];
                let title = get![args;1];
                let event = if !class.is_empty() && !title.is_empty() {
                    Event::ActiveWindowChangedV1(Some((class, title)))
                } else {
                    Event::ActiveWindowChangedV1(None)
                };

                Ok(event)
            }
            ParsedEventType::ActiveWindowChangedV2 => {
                let addr = get![ref args;0];
                let event = if !addr.is_empty() {
                    Event::ActiveWindowChangedV2(Some(Address::new(addr)))
                } else {
                    Event::ActiveWindowChangedV2(None)
                };
                Ok(event)
            }
            ParsedEventType::FullscreenStateChanged => {
                Ok(Event::FullscreenStateChanged(get![ref args;0] != "0"))
            }
            ParsedEventType::MonitorRemoved => Ok(Event::MonitorRemoved(get![args;0])),
            ParsedEventType::MonitorAddedV2 => Ok(Event::MonitorAdded(MonitorAddedEventData {
                id: parse_int!(get![ref args;0], event: "MonitorAddedV2" => MonitorId),
                name: get![args;1],
                description: get![args;2],
            })),
            ParsedEventType::WindowOpened => Ok(Event::WindowOpened(WindowOpenEvent {
                window_address: Address::new(get![ref args;0]),
                workspace_name: get![args;1],
                window_class: get![args;2],
                window_title: get![args;3],
            })),
            ParsedEventType::WindowClosed => Ok(Event::WindowClosed(Address::new(get![args;0]))),
            ParsedEventType::WindowMovedV2 => Ok(Event::WindowMoved(WindowMoveEvent {
                window_address: Address::fmt_new(get![ref args;0]),
                workspace_id: parse_int!(get![ref args;1], event: "WindowMoved"),
                workspace_name: parse_string_as_work(get![args;2]),
            })),
            ParsedEventType::ActiveSpecial => {
                let workspace_name = get![args;0];
                let monitor_name = get![args;1];
                if workspace_name.is_empty() {
                    Ok(Event::SpecialRemoved(monitor_name))
                } else {
                    Ok(Event::ChangedSpecial(ChangedSpecialEventData {
                        monitor_name,
                        workspace_name,
                    }))
                }
            }
            ParsedEventType::LayoutChanged => Ok(Event::LayoutChanged(LayoutEvent {
                keyboard_name: get![args;0],
                layout_name: get![args;1],
            })),
            ParsedEventType::SubMapChanged => Ok(Event::SubMapChanged(get![args;0])),
            ParsedEventType::LayerOpened => Ok(Event::LayerOpened(get![args;0])),
            ParsedEventType::LayerClosed => Ok(Event::LayerClosed(get![args;0])),
            ParsedEventType::FloatStateChanged => {
                let state = get![ref args;1] == "0"; // FIXME: does 0 mean it's floating?
                Ok(Event::FloatStateChanged(WindowFloatEventData {
                    address: Address::new(get![ref args;0]),
                    floating: state,
                }))
            }
            ParsedEventType::Screencast => {
                let state = get![ref args;0] == "1";
                let owner = get![ref args;1] == "1";
                Ok(Event::Screencast(ScreencastEventData {
                    turning_on: state,
                    monitor: owner,
                }))
            }
            ParsedEventType::UrgentStateChanged => {
                Ok(Event::UrgentStateChanged(Address::new(get![ref args;0])))
            }
            ParsedEventType::WindowTitleChangedV2 => {
                Ok(Event::WindowTitleChanged(WindowTitleEventData {
                    address: Address::new(get![ref args;0]),
                    title: get![args;1],
                }))
            }
            ParsedEventType::ConfigReloaded => Ok(Event::ConfigReloaded),
            ParsedEventType::IgnoreGroupLock => {
                Ok(Event::IgnoreGroupLockStateChanged(get![ref args;0] == "1"))
            }
            ParsedEventType::LockGroups => {
                Ok(Event::LockGroupsStateChanged(get![ref args;0] == "1"))
            }
            ParsedEventType::Pin => Ok(Event::WindowPinned(WindowPinEventData {
                address: Address::new(get![ref args;0]),
                pinned: get![ref args;1] == "1",
            })),
            ParsedEventType::ToggleGroup => Ok(Event::GroupToggled(GroupToggledEventData {
                toggled: get![ref args;0] == "1",
                window_addresses: get![ref args;1].split(",").map(Address::new).collect(),
            })),
            ParsedEventType::MoveIntoGroup => {
                Ok(Event::WindowMovedIntoGroup(Address::new(get![ref args;0])))
            }
            ParsedEventType::MoveOutOfGroup => {
                Ok(Event::WindowMovedOutOfGroup(Address::new(get![ref args;0])))
            }
        },
    });

    let mut events: Vec<Event> = Vec::new();

    for event in parsed_events {
        events.push(event?);
    }

    Ok(events)
}
