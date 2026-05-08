use crate::default_instance;
use crate::instance::Instance;
use crate::lua::{format_bool_field, format_string, format_string_field, format_string_field_opt};
use crate::shared::*;
use derive_more::Display;
use std::string::ToString;

/// This enum is for identifying a window
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum WindowIdentifier {
    /// A Regular Expression to match the window class (handled by Hyprland)
    #[display("class:{_0}")]
    ClassRegularExpression(String),
    /// A Regular Expression to match the initial window class (handled by Hyprland)
    #[display("initialclass:{_0}")]
    InitialClassRegularExpression(String),
    /// A Regular Expression to match the window title (handled by Hyprland)
    #[display("title:{_0}")]
    TitleRegularExpression(String),
    /// A Regular Expression to match the intial window title (handled by Hyprland)
    #[display("initialtitle:{_0}")]
    InitialTitleRegularExpression(String),
    /// A Regular Expression to match a window tag (handled by Hyprland)
    #[display("tag:{_0}")]
    TagRegularExpression(String),
    /// The window's process Id
    #[display("pid:{_0}")]
    ProcessId(u32),
    /// The address of a window
    #[display("address:{_0}")]
    Address(Address),
    /// The active window
    #[display("activewindow")]
    ActiveWindow,
    /// The first floating window
    #[display("floating")]
    Floating,
    /// The first tiled window
    #[display("tiled")]
    Tiled,
}

/// This enum is used for identifying monitors
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum MonitorIdentifier {
    /// The monitor that is to the specified direction of the active one
    #[display("{_0}")]
    Direction(Direction),
    /// The monitor id
    #[display("{_0}")]
    Id(MonitorId),
    /// The monitor name
    #[display("{_0}")]
    Name(String),
    /// The current monitor
    #[display("current")]
    Current,
    /// The workspace relative to the current workspace
    #[display("{}", format_relative(*_0))]
    Relative(i32),
    /// The workspace with this description
    #[display("desc:{_0}")]
    Description(String),
}

/// This enum is for identifying workspaces that also includes the special workspace
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum WorkspaceIdentifier {
    /// The workspace Id
    Id(WorkspaceId),
    /// The workspace relative to the current workspace
    #[display("{}", format_relative(*_0))]
    Relative(i32),

    /// The workspace on the monitor relative to the current workspace
    #[display("m{}", format_relative(*_0))]
    RelativeOnMonitor(i32),
    /// The workspace on the monitor absolute
    #[display("m~{}", *_0)]
    AbsoluteOnMonitor(u32),
    /// The workspace on the monitor relative to the current workspace, including empty workspaces
    #[display("r{}", format_relative(*_0))]
    RelativeOnMonitorIncludingEmpty(i32),
    /// The workspace on the monitor absolute, including empty workspaces
    #[display("r~{}", *_0)]
    AbsoluteOnMonitorIncludingEmpty(u32),
    /// The open workspace relative to the current workspace
    #[display("e{}", format_relative(*_0))]
    RelativeOpen(i32),
    /// The open workspace absolute
    #[display("e~{}", *_0)]
    AbsoluteOpen(u32),

    /// The name of the workspace
    #[display("name:{_0}")]
    Name(String),
    /// The previous Workspace
    #[display("previous")]
    Previous,
    /// The previous Workspace
    #[display("previous_per_monitor")]
    PreviousPerMonitor,

    /// The first available empty workspace
    #[display("{}", format!("empty{}", _0))]
    Empty(FirstEmpty),
    /// The special workspace
    #[display("special{}", format_special_workspace_ident(_0))]
    Special(Option<String>),
}

/// This struct holds options for the first empty workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{{ on_monitor = {on_monitor}, next = {next} }}")]
pub struct FirstEmpty {
    /// If the first empty workspace should be on the monitor
    pub on_monitor: bool,
    /// If the first empty workspace should be next
    pub next: bool,
}

/// This enum holds directions, typically used for moving
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Direction {
    #[display("u")]
    Up,
    #[display("d")]
    Down,
    #[display("r")]
    Right,
    #[display("l")]
    Left,
}

fn format_relative(int: i32) -> String {
    match int {
        0 => "+0".to_owned(),
        i if i > 0 => format!("+{i}"),
        _ => format!("-{int}"),
    }
}
fn format_special_workspace_ident(opt: &Option<String>) -> String {
    match opt {
        Some(o) => ":".to_owned() + o,
        None => String::new(),
    }
}

// /// This enum displays tje different fullscreen modes
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
// pub enum FullscreenState {
//     #[display("0")]
//     None,
//     #[display("1")]
//     Maximize,
//     #[display("2")]
//     Fullscreen,
//     #[display("3")]
//     MaximizeFullscreen,
// }

pub trait ToDispatch: ToString {}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Dispatch {
    /// This lets you use dispatchers not supported by hyprland-rs yet or for custom lua functions.
    /// Raw string will be used as the dispatcher
    #[display("{}", _0)]
    Unimplemented(String),
    /// execute a command. Rules can be a table of window rule effects to apply.
    /// TODO implement rules
    #[display("hl.dsp.exec_cmd({})", format_string(_0))]
    ExecCmd(String, Option<()>),
    /// execute a raw command. While exec_cmd will do bash -c, this won’t.
    #[display("hl.dsp.exec_raw({})", format_string(_0))]
    ExecRaw(String),
    /// move the focus in a direction
    #[display("hl.dsp.focus({{ {}}})", format_string_field("direction", &_0.to_string()))]
    FocusDirection(Direction),
    /// move the focus to a monitor
    #[display("hl.dsp.focus({{ {}}})", format_string_field("monitor", &_0.to_string()))]
    FocusMonitor(MonitorIdentifier),
    /// move the focus to a workspace (2. = on_current_monitor)
    #[display(
        "hl.dsp.focus({{ {} {}}})",
        format_string_field("workspace", &_0.to_string()),
        format_bool_field("on_current_monitor", *_1)
    )]
    FocusWorkspace(WorkspaceIdentifier, bool),
    /// move the focus to a window
    #[display("hl.dsp.focus({{ {}}})", format_string_field("window", &_0.to_string()))]
    FocusWindow(WindowIdentifier),
    /// move the focus to an urgent, or last window
    #[display("hl.dsp.focus({{ urgent_or_last }})")]
    FocusUrgentOrLast,
    /// move the focus to the last window
    #[display("hl.dsp.focus({{ last }})")]
    FocusLast,
    /// quit Hyprland. It’s recommended to use hyprshutdown instead of this.
    #[display("hl.dsp.exit()")]
    Exit,
    /// move to a submap
    #[display("hl.dsp.submap({})", format_string(_0))]
    SubMap(String),
    /// pass the shortcut to a window
    #[display("hl.dsp.pass({{ {}}})", format_string_field_opt("window", &_0))]
    Pass(Option<WindowIdentifier>),

    // send a specific shortcut to a window
    // send_shortcut({ mods, key, window? })
    // same as above, but you control down / up
    // send_key_state({ mods, key, state, window? })
    /// send a layout message as a string
    #[display("hl.dsp.layout({})", format_string(_0))]
    Layout(String),

    // toggle monitors on/off (not physically, as in idle-screensaver.)
    // dpms({ action?, monitor? })
    /// send an event to socket2.
    #[display("hl.dsp.event({})", format_string(_0))]
    Event(String),
    /// activate a dbus global shortcut. See https://wiki.hypr.land/Configuring/Basics/Binds/#DBus-Global-Shortcuts
    #[display("hl.dsp.global({})", format_string(_0))]
    Global(String),
    /// sets elapsed time for all idle timers, ignoring idle inhibitors. Timers return to normal behavior upon the next activity. Do not use with a keybind directly.
    #[display("hl.dsp.force_idle({})", _0)]
    ForceIdle(u64),
    /// does nothing. Useful for conditional binds.
    #[display("hl.dsp.no_op()")]
    NoOp(),
}
impl Dispatch {
    /// This function sets a keyword's value
    pub fn apply(&self) -> crate::Result<()> {
        self.instance_apply(default_instance()?)
    }

    /// This function sets a keyword's value
    pub fn instance_apply(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        let ret = instance.write_to_socket(command!(Empty, "eval hl.dispatch({})", lua))?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not dispatch: {}",
                ret
            )));
        }
        Ok(())
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn apply_async(&self) -> crate::Result<()> {
        self.instance_apply_async(default_instance()?).await
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_apply_async(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        let ret = instance
            .write_to_socket_async(command!(Empty, "eval hl.dispatch({})", lua))
            .await?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not dispatch: {}",
                ret
            )));
        }
        Ok(())
    }
}
impl ToDispatch for Dispatch {}

mod windows {
    use crate::dispatch_new::{ToDispatch, WindowIdentifier};
    use derive_more::Display;

    #[derive(Debug, Clone, PartialEq, Eq, Display)]
    pub enum Dispatch {
        /// Close a window.
        #[display("todo")]
        Close(Option<WindowIdentifier>),
        /// Kill a window
        #[display("todo")]
        Kill(Option<WindowIdentifier>),
        // ...
    }
    impl ToDispatch for Dispatch {}
}
