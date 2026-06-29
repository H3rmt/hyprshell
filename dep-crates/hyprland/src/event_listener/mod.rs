//! # Event Listener Module
//! for documentation go to:
//! * [EventStream] for the event listener implementation based on the [futures_lite::Stream] api
//! * [EventListener] for the normal [Fn] based event listener
//! * [AsyncEventListener] for the [Fn] based event listener which uses closures that return [std::future::Future]s

#[macro_use]
mod macros;

mod shared;
pub use crate::event_listener::shared::*;

mod immutable;
pub use crate::event_listener::immutable::EventListener;

#[cfg(any(feature = "async-lite", feature = "tokio"))]
mod async_im;
#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub use crate::event_listener::async_im::AsyncEventListener;

#[cfg(any(feature = "async-lite", feature = "tokio"))]
mod stream;
#[cfg(any(feature = "async-lite", feature = "tokio"))]
pub use crate::event_listener::stream::EventStream;
use crate::shared::Address;

// generates code for the closure based event listeners
events! {
    WorkspaceChanged => WorkspaceEventData, "on workspace change", "changed workspace to" => id;
    WorkspaceAdded => WorkspaceEventData, "a workspace is created", "workspace was added" => id;
    WorkspaceDeleted => WorkspaceEventData, "a workspace is destroyed", "a workspace was destroyed" => data;
    WorkspaceMoved => WorkspaceMovedEventData, "a workspace is moved", "workspace was moved" => id;
    WorkspaceRenamed => NonSpecialWorkspaceEventData, "a workspace is renamed", "workspace was renamed" => id;
    ActiveMonitorChanged => MonitorEventData, "the active monitor is changed", "Active monitor changed to" => data;
    ActiveWindowChanged => Option<WindowEventData>, "the active window is changed", "Active window changed" => data;
    FullscreenStateChanged => bool, "the fullscreen state is changed", "Fullscreen is on" => state;
    MonitorAdded => MonitorAddedEventData, "a new monitor is added", "Monitor added" => data;
    MonitorRemoved => String, "a monitor is removed", "Monitor removed" => data;
    WindowOpened => WindowOpenEvent, "a window is opened", "Window opened" => data;
    WindowClosed => Address, "a window is closed", "Window closed" => data;
    WindowMoved => WindowMoveEvent, "a window is moved", "Window moved" => data;
    SpecialRemoved => String, "a monitor's special workspace is removed", "Special Workspace removed" => monitor;
    ChangedSpecial => ChangedSpecialEventData, "a monitor's special workspace is changed", "Special Workspace changed" => data;
    LayoutChanged => LayoutEvent, "the keyboard layout is changed", "Layout changed" => data;
    SubMapChanged => String, "the submap is changed", "Submap changed" => data;
    LayerOpened => String, "a new layer is opened", "Layer opened" => data;
    LayerClosed => String, "a layer is closed", "Layer closed" => data;
    FloatStateChanged => WindowFloatEventData, "the float state of a window is changed", "Float state changed" => data;
    UrgentStateChanged => Address, "the urgent state of a window is changed", "urgent state changed" => data;
    WindowTitleChanged => WindowTitleEventData, "a window title is changed", "A window title changed" => data;
    Screencast => ScreencastEventData, "the screencast state of a window is changed", "screencast state changed" => data;
    ConfigReloaded => (), "the configuration of Hyprland is reloaded", "config reloaded" => _empty;
    IgnoreGroupLockStateChanged => bool, "the state of ignore group lock is toggled", "ignore group lock toggled to" => data;
    LockGroupsStateChanged => bool, "the state of lock groups is toggled", "lock group state toggled to" => data;
    WindowPinned => WindowPinEventData, "the pinned state of a window is changed", "window pin was set to" => state;
    GroupToggled => GroupToggledEventData, "a group was toggled", "the group toggle state was set to" => data;
    WindowMovedIntoGroup => Address, "a window was moved into a group", "a window was moved into a group with the address of" => addr;
    WindowMovedOutOfGroup => Address, "a window was moved out of a group", "a window was moved out of a group with the address of" => addr;
    Unknown => UnknownEventData, "the state of some unknown event changed", "unknown state changed to" => value
}
