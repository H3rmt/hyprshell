use derive_more::{Constructor, Display as MDisplay};
use std::fmt::Display as FDisplay;

use crate::default_instance;
use crate::instance::Instance;
use crate::shared::*;

/// Reload hyprland config
pub mod reload {
    use super::*;

    /// Reload hyprland config
    pub fn call() -> crate::Result<()> {
        instance_call(default_instance()?)
    }

    /// Reload hyprland config
    pub fn instance_call(instance: &Instance) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "reload"))?;
        Ok(())
    }

    /// Reload hyprland config (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async() -> crate::Result<()> {
        instance_call_async(default_instance()?).await
    }

    /// Reload hyprland config (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(instance: &Instance) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "reload"))
            .await?;
        Ok(())
    }
}
/// Enter kill mode (similar to xkill)
pub mod kill {
    use super::*;

    /// Enter kill mode (similar to xkill)
    pub fn call() -> crate::Result<()> {
        instance_call(default_instance()?)
    }

    /// Enter kill mode (similar to xkill)
    pub fn instance_call(instance: &Instance) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "kill"))?;
        Ok(())
    }

    /// Enter kill mode (similar to xkill) (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async() -> crate::Result<()> {
        instance_call_async(default_instance()?).await
    }

    /// Enter kill mode (similar to xkill) (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(instance: &Instance) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "kill"))
            .await?;
        Ok(())
    }
}

/// Set the cursor theme
pub mod set_cursor {
    use super::*;

    /// Set the cursor theme
    pub fn call<Str: FDisplay>(theme: Str, size: u16) -> crate::Result<()> {
        instance_call(default_instance()?, theme, size)
    }

    /// Set the cursor theme
    pub fn instance_call<Str: FDisplay>(
        instance: &Instance,
        theme: Str,
        size: u16,
    ) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "setcursor {theme} {size}"))?;
        Ok(())
    }

    /// Set the cursor theme (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async<Str: FDisplay>(theme: Str, size: u16) -> crate::Result<()> {
        instance_call_async(default_instance()?, theme, size).await
    }

    /// Set the cursor theme (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async<Str: FDisplay>(
        instance: &Instance,
        theme: Str,
        size: u16,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "setcursor {theme} {size}"))
            .await?;
        Ok(())
    }
}

/// Stuff related to managing virtual outputs/displays
pub mod output {
    use super::*;

    /// Output backend types
    #[derive(Debug, MDisplay, Clone, Copy, PartialEq, Eq)]
    pub enum OutputBackends {
        /// The wayland output backend
        #[display("wayland")]
        Wayland,
        /// The x11 output backend
        #[display("x11")]
        X11,
        /// The headless output backend
        #[display("headless")]
        Headless,
        /// Let Hyprland decide the backend type
        #[display("auto")]
        Auto,
    }

    /// Create virtual displays
    pub fn create(backend: OutputBackends, name: Option<&str>) -> crate::Result<()> {
        instance_create(default_instance()?, backend, name)
    }

    /// Remove virtual displays
    pub fn remove<Str: FDisplay>(name: Str) -> crate::Result<()> {
        instance_remove(default_instance()?, name)
    }

    /// Create virtual displays
    pub fn instance_create(
        instance: &Instance,
        backend: OutputBackends,
        name: Option<&str>,
    ) -> crate::Result<()> {
        let name = name.unwrap_or_default();
        instance.write_to_socket(command!(Empty, "output create {backend} {name}"))?;
        Ok(())
    }

    /// Remove virtual displays
    pub fn instance_remove<Str: FDisplay>(instance: &Instance, name: Str) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "output remove {name}"))?;
        Ok(())
    }

    /// Create virtual displays
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn create_async(backend: OutputBackends, name: Option<&str>) -> crate::Result<()> {
        instance_create_async(default_instance()?, backend, name).await
    }

    /// Create virtual displays
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_create_async(
        instance: &Instance,
        backend: OutputBackends,
        name: Option<&str>,
    ) -> crate::Result<()> {
        let name = name.unwrap_or_default();
        instance
            .write_to_socket_async(command!(Empty, "output create {backend} {name}"))
            .await?;
        Ok(())
    }

    /// Remove virtual displays
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn remove_async<Str: FDisplay>(name: Str) -> crate::Result<()> {
        instance_remove_async(default_instance()?, name).await
    }

    /// Remove virtual displays
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_remove_async<Str: FDisplay>(
        instance: &Instance,
        name: Str,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "output remove {name}"))
            .await?;
        Ok(())
    }
}

/// Switch the xkb layout index for a keyboard
pub mod switch_xkb_layout {
    use super::*;

    /// The types of Cmds used by [switch_xkb_layout]
    #[derive(Debug, MDisplay, Clone, Copy, PartialEq, Eq)]
    pub enum SwitchXKBLayoutCmdTypes {
        /// Next input
        #[display("next")]
        Next,
        /// Previous inout
        #[display("prev")]
        Previous,
        /// Set to a specific input id
        #[display("{_0}")]
        Id(u8),
    }

    /// Switch the xkb layout index for a keyboard
    pub fn call<Str: FDisplay>(device: Str, cmd: SwitchXKBLayoutCmdTypes) -> crate::Result<()> {
        instance_call(default_instance()?, device, cmd)
    }

    /// Switch the xkb layout index for a keyboard
    pub fn instance_call<Str: FDisplay>(
        instance: &Instance,
        device: Str,
        cmd: SwitchXKBLayoutCmdTypes,
    ) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "switchxkblayout {device} {cmd}"))?;
        Ok(())
    }

    /// Switch the xkb layout index for a keyboard
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async<Str: FDisplay>(
        instance: &Instance,
        device: Str,
        cmd: SwitchXKBLayoutCmdTypes,
    ) -> crate::Result<()> {
        instance_call_async(instance, device, cmd).await
    }

    /// Switch the xkb layout index for a keyboard
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async<Str: FDisplay>(
        instance: &Instance,
        device: Str,
        cmd: SwitchXKBLayoutCmdTypes,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "switchxkblayout {device} {cmd}"))
            .await?;
        Ok(())
    }
}

/// Creates a error that Hyprland will display
pub mod set_error {
    use super::*;

    /// Creates a error that Hyprland will display
    pub fn call(color: Color, msg: String) -> crate::Result<()> {
        instance_call(default_instance()?, color, msg)
    }

    /// Creates a error that Hyprland will display
    pub fn instance_call(instance: &Instance, color: Color, msg: String) -> crate::Result<()> {
        instance.write_to_socket(command!(Empty, "seterror {color} {msg}"))?;
        Ok(())
    }

    /// Creates a error that Hyprland will display (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async(color: Color, msg: String) -> crate::Result<()> {
        instance_call_async(default_instance()?, color, msg).await
    }

    /// Creates a error that Hyprland will display (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(
        instance: &Instance,
        color: Color,
        msg: String,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(Empty, "seterror {color} {msg}"))
            .await?;
        Ok(())
    }
}

/// Creates a notification with Hyprland
pub mod notify {
    use super::*;

    #[allow(missing_docs)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(i8)]
    pub enum Icon {
        NoIcon = -1,
        Warning = 0,
        Info = 1,
        Hint = 2,
        Error = 3,
        Confused = 4,
        Ok = 5,
    }

    /// Creates a notification with Hyprland
    pub fn call(
        icon: Icon,
        time: std::time::Duration,
        color: Color,
        msg: String,
    ) -> crate::Result<()> {
        instance_call(default_instance()?, icon, time, color, msg)
    }

    /// Creates a notification with Hyprland
    pub fn instance_call(
        instance: &Instance,
        icon: Icon,
        time: std::time::Duration,
        color: Color,
        msg: String,
    ) -> crate::Result<()> {
        instance.write_to_socket(command!(
            Empty,
            "notify {} {} {color} {msg}",
            icon as i8,
            time.as_millis()
        ))?;
        Ok(())
    }

    /// Creates a error that Hyprland will display (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async(
        icon: Icon,
        time: std::time::Duration,
        color: Color,
        msg: String,
    ) -> crate::Result<()> {
        instance_call_async(default_instance()?, icon, time, color, msg).await
    }

    /// Creates a error that Hyprland will display (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(
        instance: &Instance,
        icon: Icon,
        time: std::time::Duration,
        color: Color,
        msg: String,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(
                Empty,
                "notify {} {} {color} {msg}",
                icon as i8,
                time.as_millis()
            ))
            .await?;
        Ok(())
    }
}
/// Dismisses all or up to a specified amount of notifications with Hyprland
pub mod dismissnotify {
    use super::*;

    /// Dismisses notifications with Hyprland
    ///
    /// If `amount` is [None] then will dismiss ALL notifications
    pub fn call(amount: Option<std::num::NonZeroU8>) -> crate::Result<()> {
        instance_call(default_instance()?, amount)
    }

    /// Dismisses notifications with Hyprland
    ///
    /// If `amount` is [None] then will dismiss ALL notifications
    pub fn instance_call(
        instance: &Instance,
        amount: Option<std::num::NonZeroU8>,
    ) -> crate::Result<()> {
        instance.write_to_socket(command!(
            Empty,
            "dismissnotify {}",
            if let Some(amount) = amount {
                amount.to_string()
            } else {
                (-1).to_string()
            }
        ))?;
        Ok(())
    }

    /// Dismisses notifications with Hyprland (async)
    ///
    /// If `amount` is [None] then will dismiss ALL notifications
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async(amount: Option<std::num::NonZeroU8>) -> crate::Result<()> {
        instance_call_async(default_instance()?, amount).await
    }

    /// Dismisses notifications with Hyprland (async)
    ///
    /// If `amount` is [None] then will dismiss ALL notifications
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(
        instance: &Instance,
        amount: Option<std::num::NonZeroU8>,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(
                Empty,
                "dismissnotify {}",
                if let Some(amount) = amount {
                    amount.to_string()
                } else {
                    (-1).to_string()
                }
            ))
            .await?;
        Ok(())
    }
}

/// A 8-bit color with a alpha channel
#[derive(Debug, Copy, Clone, MDisplay, Constructor, PartialEq, Eq)]
#[display("rgba({_0:02x}{_1:02x}{_2:02x}{_3:02x})")]
pub struct Color(u8, u8, u8, u8);

/// Provides things to setting props
pub mod set_prop {
    use super::*;

    fn l(b: bool) -> &'static str {
        if b { "lock" } else { "" }
    }

    /// Type that represents a prop
    #[derive(MDisplay, Clone, PartialEq)]
    pub enum PropType {
        /// The animation style
        #[display("animationstyle {_0}")]
        AnimationStyle(String),
        /// The roundness
        #[display("rounding {_0} {}", l(*_1))]
        Rounding(
            i64,
            /// locked
            bool,
        ),
        /// Force no blur
        #[display("forcenoblur {} {}", *_0 as u8, l(*_1))]
        ForceNoBlur(
            bool,
            /// locked
            bool,
        ),
        /// Force opaque
        #[display("forceopaque {} {}", *_0 as u8, l(*_1))]
        ForceOpaque(
            bool,
            /// locked
            bool,
        ),
        /// Force opaque overriden
        #[display("forceopaqueoverriden {} {}", *_0 as u8, l(*_1))]
        ForceOpaqueOverriden(
            bool,
            /// locked
            bool,
        ),
        /// Force allow input
        #[display("forceallowsinput {} {}", *_0 as u8, l(*_1))]
        ForceAllowsInput(
            bool,
            /// locked
            bool,
        ),
        /// Force no animations
        #[display("forcenoanims {} {}", *_0 as u8, l(*_1))]
        ForceNoAnims(
            bool,
            /// locked
            bool,
        ),
        /// Force no border
        #[display("forcenoborder {} {}", *_0 as u8, l(*_1))]
        ForceNoBorder(
            bool,
            /// locked
            bool,
        ),
        /// Force no shadow
        #[display("forcenoshadow {} {}", *_0 as u8, l(*_1))]
        ForceNoShadow(
            bool,
            /// locked
            bool,
        ),
        /// Allow for windoe dancing?
        #[display("windowdancecompat {} {}", *_0 as u8, l(*_1))]
        WindowDanceCompat(
            bool,
            /// locked
            bool,
        ),
        /// Allow for overstepping max size
        #[display("nomaxsize {} {}", *_0 as u8, l(*_1))]
        NoMaxSize(
            bool,
            /// locked
            bool,
        ),
        /// Dim around?
        #[display("dimaround {} {}", *_0 as u8, l(*_1))]
        DimAround(
            bool,
            /// locked
            bool,
        ),
        /// Makes the next setting be override instead of multiply
        #[display("alphaoverride {} {}", *_0 as u8, l(*_1))]
        AlphaOverride(
            bool,
            /// locked
            bool,
        ),
        /// The alpha
        #[display("alpha {_0} {}", l(*_1))]
        Alpha(
            f32,
            /// locked
            bool,
        ),
        /// Makes the next setting be override instead of multiply
        #[display("alphainactiveoverride {} {}", *_0 as u8, l(*_1))]
        AlphaInactiveOverride(
            bool,
            /// locked
            bool,
        ),
        /// The alpha for inactive
        #[display("alphainactive {_0} {}", l(*_1))]
        AlphaInactive(
            f32,
            /// locked
            bool,
        ),
        /// The active border color
        #[display("alphabordercolor {_0} {}", l(*_1))]
        ActiveBorderColor(
            Color,
            /// locked
            bool,
        ),
        /// The inactive border color
        #[display("inalphabordercolor {_0} {}", l(*_1))]
        InactiveBorderColor(
            Color,
            /// locked
            bool,
        ),
    }

    /// Sets a window prob
    pub fn call(ident: String, prop: PropType, lock: bool) -> crate::Result<()> {
        instance_call(default_instance()?, ident, prop, lock)
    }

    /// Sets a window prob
    pub fn instance_call(
        instance: &Instance,
        ident: String,
        prop: PropType,
        lock: bool,
    ) -> crate::Result<()> {
        instance.write_to_socket(command!(
            Empty,
            "setprop {ident} {prop} {}",
            if lock { "lock" } else { "" }
        ))?;
        Ok(())
    }

    /// Sets a window prob (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async(ident: String, prop: PropType, lock: bool) -> crate::Result<()> {
        instance_call_async(default_instance()?, ident, prop, lock).await
    }

    /// Sets a window prob (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(
        instance: &Instance,
        ident: String,
        prop: PropType,
        lock: bool,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(
                Empty,
                "setprop {ident} {prop} {}",
                if lock { "lock" } else { "" }
            ))
            .await?;
        Ok(())
    }
}

/// Provides functions for communication with plugin system
pub mod plugin {
    use super::*;
    use crate::error::HyprError;
    use std::path::Path;

    /// This struct represents a loaded plugin
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct Plugin {
        /// plugin name
        pub name: String,
        /// plugin author
        pub author: String,
        /// handle to plugin
        pub handle: String,
        /// plugin version
        pub version: String,
        /// plugin description
        pub description: String,
    }

    /// Returns a list of all plugins
    pub fn list() -> crate::Result<Vec<Plugin>> {
        instance_list(default_instance()?)
    }

    /// Returns a list of all plugins
    pub fn instance_list(instance: &Instance) -> crate::Result<Vec<Plugin>> {
        let data = instance.write_to_socket(command!(JSON, "plugin list"))?;
        let deserialized: Vec<Plugin> = serde_json::from_str(&data)?;
        Ok(deserialized)
    }

    /// Returns a list of all plugins (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn list_async() -> crate::Result<Vec<Plugin>> {
        instance_list_async(default_instance()?).await
    }

    /// Returns a list of all plugins (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_list_async(instance: &Instance) -> crate::Result<Vec<Plugin>> {
        let data = instance
            .write_to_socket_async(command!(JSON, "plugin list"))
            .await?;
        let deserialized: Vec<Plugin> = serde_json::from_str(&data)?;
        Ok(deserialized)
    }

    /// Loads a plugin, by absolute path
    pub fn load(path: &Path) -> crate::Result<()> {
        instance_load(default_instance()?, path)
    }

    /// Loads a plugin, by absolute path
    pub fn instance_load(instance: &Instance, path: &Path) -> crate::Result<()> {
        let str = instance.write_to_socket(command!(Empty, "plugin load {}", path.display()))?;
        if str.contains("could not be loaded") {
            Err(HyprError::Internal(str))
        } else {
            Ok(())
        }
    }

    /// Loads a plugin, by absolute path (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn load_async(path: &Path) -> crate::Result<()> {
        instance_load_async(default_instance()?, path).await
    }

    /// Loads a plugin, by absolute path (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_load_async(instance: &Instance, path: &Path) -> crate::Result<()> {
        let str = instance
            .write_to_socket_async(command!(Empty, "plugin load {}", path.display()))
            .await?;
        if str.contains("could not be loaded") {
            Err(HyprError::Internal(str))
        } else {
            Ok(())
        }
    }

    /// Unloads a plugin, by absolute path.
    pub fn unload(path: &Path) -> crate::Result<()> {
        instance_unload(default_instance()?, path)
    }

    /// Unloads a plugin, by absolute path.
    pub fn instance_unload(instance: &Instance, path: &Path) -> crate::Result<()> {
        let str = instance.write_to_socket(command!(Empty, "plugin unload {}", path.display()))?;
        if str.contains("plugin not loaded") {
            Err(HyprError::Internal(str))
        } else {
            Ok(())
        }
    }

    /// Unloads a plugin, by absolute path (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn unload_async(path: &Path) -> crate::Result<()> {
        instance_unload_async(default_instance()?, path).await
    }

    /// Unloads a plugin, by absolute path (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_unload_async(instance: &Instance, path: &Path) -> crate::Result<()> {
        let str = instance
            .write_to_socket_async(command!(Empty, "plugin unload {}", path.display()))
            .await?;
        if str.contains("plugin not loaded") {
            Err(HyprError::Internal(str))
        } else {
            Ok(())
        }
    }
}

/// This module allows listing running hyprland instances
pub mod instance {
    use crate::shared::get_hypr_path;
    use std::fs::{DirEntry, File};
    use std::io::Read;
    use std::path::Path;

    /// This struct represents a running Hyprland instance
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct Instance {
        /// instance name (9958d29...) in /run/user/$UID/hypr/$instance
        pub instance: String,
        /// ???
        pub time: u64,
        /// pid of hyprland process
        pub pid: u32,
        /// name of wayland socket in /run/user/$UID/$wl_socket
        pub wl_socket: String,
    }

    /// Returns a list of running instances
    pub fn instance_list() -> crate::Result<Vec<Instance>> {
        let buf = get_hypr_path()?;
        let entries = std::fs::read_dir(buf)?;
        let mut instances = Vec::new();
        for entry in entries.flatten() {
            if let Some(instance) = parse_instance_entry(entry) {
                instances.push(instance);
            }
        }
        instances.retain(|el| Path::new(&format!("/proc/{}", el.pid)).exists());
        Ok(instances)
    }

    fn parse_instance_entry(entry: DirEntry) -> Option<Instance> {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let first = file_name.find('_')?;
        let last = file_name.rfind('_')?;
        if last <= first {
            return None;
        }
        let time = file_name[first + 1..last].parse::<u64>().ok()?;

        let lock_path = entry.path().join("hyprland.lock");
        let mut file = File::open(&lock_path).ok()?;
        if file.metadata().ok()?.len() == 0 {
            return None; // Empty lock file, skip this instance
        }
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        let data = content
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();
        if data.len() != 2 {
            return None;
        }

        let pid = data.first().and_then(|s| s.parse::<u32>().ok())?;
        let wl_socket = data.get(1).cloned().unwrap_or_default();

        Some(Instance {
            instance: file_name,
            time,
            pid,
            wl_socket,
        })
    }
}
