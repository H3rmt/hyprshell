use super::*;
use crate::default_instance;
use crate::error::hypr_err;
use crate::instance::Instance;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// This pub(crate) enum holds every socket command that returns data
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataCommands {
    #[display("monitors all")]
    Monitors,
    #[display("workspaces")]
    Workspaces,
    #[display("activeworkspace")]
    ActiveWorkspace,
    #[display("clients")]
    Clients,
    #[display("activewindow")]
    ActiveWindow,
    #[display("layers")]
    Layers,
    #[display("devices")]
    Devices,
    #[display("version")]
    Version,
    #[display("cursorpos")]
    CursorPosition,
    #[display("binds")]
    Binds,
    #[display("animations")]
    Animations,
    #[display("workspacerules")]
    WorkspaceRules,
}

/// This struct holds a basic identifier for a workspace often used in other structs
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBasic {
    /// The workspace Id
    pub id: WorkspaceId,
    /// The workspace's name
    pub name: String,
}

/// This enum provides the different monitor transforms
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum Transforms {
    /// No transform
    Normal = 0,
    /// Rotated 90 degrees
    Normal90 = 1,
    /// Rotated 180 degrees
    Normal180 = 2,
    /// Rotated 270 degrees
    Normal270 = 3,
    /// Flipped
    Flipped = 4,
    /// Flipped and rotated 90 degrees
    Flipped90 = 5,
    /// Flipped and rotated 180 degrees
    Flipped180 = 6,
    /// Flipped and rotated 270 degrees
    Flipped270 = 7,
}

/// This struct holds information for a monitor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Monitor {
    /// The monitor id
    pub id: MonitorId,
    /// The monitor's name
    pub name: String,
    /// The monitor's description
    pub description: String,
    /// The monitor width (in pixels)
    pub width: u16,
    /// The monitor height (in pixels)
    pub height: u16,
    /// The monitor's refresh rate (in hertz)
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f32,
    /// The monitor's position on the x axis (not irl ofc)
    pub x: i32,
    /// The monitor's position on the x axis (not irl ofc)
    pub y: i32,
    /// A basic identifier for the active workspace
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WorkspaceBasic,
    /// A basic identifier for the special workspace
    #[serde(rename = "specialWorkspace")]
    pub special_workspace: WorkspaceBasic,
    /// Reserved is the amount of space (in pre-scale pixels) that a layer surface has claimed
    pub reserved: (u16, u16, u16, u16),
    /// The display's scale
    pub scale: f32,
    /// I think like the rotation?
    pub transform: Transforms,
    /// a string that identifies if the display is active
    pub focused: bool,
    /// The dpms status of a monitor
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,
    /// VRR state
    pub vrr: bool,
    /// Is the monitor disabled or not
    pub disabled: bool,
    /// The physical width of the monitor in mm
    #[serde(rename = "physicalWidth", default)]
    pub physical_width: u16,
    /// The physical size of the monitor in mm
    #[serde(rename = "physicalHeight", default)]
    pub physical_height: u16,
}

impl HyprDataActive for Monitor {
    fn get_active() -> crate::Result<Self> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Self> {
        let all = Monitors::instance_get(instance)?;
        if let Some(it) = all.into_iter().find(|item| item.focused) {
            Ok(it)
        } else {
            hypr_err!("No active Hyprland monitor detected!")
        }
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self> {
        let all = Monitors::instance_get_async(instance).await?;
        if let Some(it) = all.into_iter().find(|item| item.focused) {
            Ok(it)
        } else {
            hypr_err!("No active Hyprland monitor detected!")
        }
    }
}

create_data_struct!(
    vector,
    name: Monitors,
    command: DataCommands::Monitors,
    holding_type: Monitor,
    doc: "This struct holds a vector of monitors"
);

/// This struct holds information for a workspace
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// The workspace Id
    pub id: WorkspaceId,
    /// The workspace's name
    pub name: String,
    /// The monitor the workspace is on
    pub monitor: String,
    /// The monitor id the workspace is on, can be None in some cases
    #[serde(rename = "monitorID")]
    pub monitor_id: Option<MonitorId>,
    /// The amount of windows in the workspace
    pub windows: u16,
    /// A bool that shows if there is a fullscreen window in the workspace
    #[serde(rename = "hasfullscreen")]
    pub fullscreen: bool,
    /// The last window's [Address]
    #[serde(rename = "lastwindow")]
    pub last_window: Address,
    /// The last window's title
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,
}

impl HyprDataActive for Workspace {
    fn get_active() -> crate::Result<Self> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Self> {
        let data = instance.write_to_socket(command!(JSON, "{}", DataCommands::ActiveWorkspace))?;
        let deserialized: Workspace = serde_json::from_str(&data)?;
        Ok(deserialized)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self> {
        let data = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::ActiveWorkspace))
            .await?;
        let deserialized: Workspace = serde_json::from_str(&data)?;
        Ok(deserialized)
    }
}

create_data_struct!(
    vector,
    name: Workspaces,
    command: DataCommands::Workspaces,
    holding_type: Workspace,
    doc: "This type provides a vector of workspaces"
);

/// This struct holds information for a client/window fullscreen mode
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum FullscreenMode {
    /// Normal window
    None = 0,
    /// Maximized window
    Maximized = 1,
    /// Fullscreen window
    Fullscreen = 2,
    /// Maximized and fullscreen window
    MaximizedFullscreen = 3,
}

/// This struct holds information for a client/window
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// The client's [`Address`][crate::shared::Address]
    pub address: Address,
    /// The window location
    pub at: (i16, i16),
    /// The window size
    pub size: (i16, i16),
    /// The workspace its on
    pub workspace: WorkspaceBasic,
    /// Is this window floating?
    pub floating: bool,
    /// The internal fullscreen mode
    pub fullscreen: FullscreenMode,
    /// The client fullscreen mode
    #[serde(rename = "fullscreenClient")]
    pub fullscreen_client: FullscreenMode,
    /// The monitor id the window is on, can be None in some cases
    pub monitor: Option<MonitorId>,
    /// The initial window class
    #[serde(rename = "initialClass")]
    pub initial_class: String,
    /// The window class
    pub class: String,
    /// The initial window title
    #[serde(rename = "initialTitle")]
    pub initial_title: String,
    /// The window title
    pub title: String,
    /// The process Id of the client
    pub pid: i32,
    /// Is this window running under XWayland?
    pub xwayland: bool,
    /// Is this window pinned?
    pub pinned: bool,
    /// Group members
    pub grouped: Vec<Box<Address>>,
    /// Is this window print on screen
    pub mapped: bool,
    /// The swallowed window
    pub swallowing: Option<Box<Address>>,
    /// When was this window last focused relatively to other windows? 0 for current, 1 previous, 2 previous before that, etc
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i8,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Empty {}

impl HyprDataActiveOptional for Client {
    fn get_active() -> crate::Result<Option<Self>> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Option<Self>> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Option<Self>> {
        let data = instance.write_to_socket(command!(JSON, "{}", DataCommands::ActiveWindow))?;
        let res = serde_json::from_str::<Empty>(&data);
        if res.is_err() {
            let t = serde_json::from_str::<Client>(&data)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Option<Self>> {
        let data = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::ActiveWindow))
            .await?;
        let res = serde_json::from_str::<Empty>(&data);
        if res.is_err() {
            let t = serde_json::from_str::<Client>(&data)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }
}

create_data_struct!(
    vector,
    name: Clients,
    command: DataCommands::Clients,
    holding_type: Client,
    doc: "This struct holds a vector of clients"
);

/// This struct holds information about a layer surface/client
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LayerClient {
    /// The layer's [`Address`][crate::shared::Address]
    pub address: Address,
    /// The layer's x position
    pub x: i32,
    /// The layer's y position
    pub y: i32,
    /// The layer's width
    pub w: i16,
    /// The layer's height
    pub h: i16,
    /// The layer's namespace
    pub namespace: String,
}

/// This struct holds all the layer surfaces for a display
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LayerDisplay {
    /// The different levels of layers
    pub levels: HashMap<String, Vec<LayerClient>>,
}

implement_iterators!(
    table,
    name: LayerDisplay,
    iterated_field: levels,
    key: String,
    value: Vec<LayerClient>,
);

create_data_struct!(
    table,
    name: Layers,
    command: DataCommands::Layers,
    key: String,
    value: LayerDisplay,
    doc: "This struct holds a hashmap of all current displays, and their layer surfaces"
);

/// This struct holds information about a mouse device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Mouse {
    /// The mouse's address
    pub address: Address,
    /// The mouse's name
    pub name: String,
}

/// This struct holds information about a keyboard device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Keyboard {
    /// The keyboard's address
    pub address: Address,
    /// The keyboard's name
    pub name: String,
    /// The keyboard rules
    pub rules: String,
    /// The keyboard model
    pub model: String,
    /// The layout of the keyboard
    pub layout: String,
    /// The keyboard variant
    pub variant: String,
    /// The keyboard options
    pub options: String,
    /// The keyboard's active keymap
    pub active_keymap: String,
    /// The keyboard's primary status
    pub main: bool,
}

/// A enum that holds the types of tablets
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabletType {
    /// The TabletPad type of tablet
    #[serde(rename = "tabletPad")]
    TabletPad,
    /// The TabletTool type of tablet
    #[serde(rename = "tabletTool")]
    TabletTool,
}

/// A enum to match what the tablet belongs to
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum TabletBelongsTo {
    /// The belongsTo data if the tablet is of type TabletPad
    TabletPad {
        /// The name of the parent
        name: String,
        /// The address of the parent
        address: Address,
    },
    /// The belongsTo data if the tablet is of type TabletTool
    Address(Address),
}

/// This struct holds information about a tablet device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tablet {
    /// The tablet's address
    pub address: Address,
    /// The tablet type
    #[serde(rename = "type")]
    pub tablet_type: Option<TabletType>,
    /// What the tablet belongs to
    #[serde(rename = "belongsTo")]
    pub belongs_to: Option<TabletBelongsTo>,
    /// The name of the tablet
    pub name: Option<String>,
}

/// This struct holds all current devices
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Devices {
    /// All the mice
    pub mice: Vec<Mouse>,
    /// All the keyboards
    pub keyboards: Vec<Keyboard>,
    /// All the tablets
    pub tablets: Vec<Tablet>,
}
impl_on!(Devices);

/// This struct holds version information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// The git branch Hyprland was built on
    pub branch: String,
    /// The git commit Hyprland was built on
    pub commit: String,
    #[serde(default)]
    /// The Hyprland version
    pub version: Option<String>,
    /// This is true if there were unstaged changed when Hyprland was built
    pub dirty: bool,
    /// The git commit message
    pub commit_message: String,
    /// The git commit date
    pub commit_date: String,
    /// The git tag hyprland was built on
    pub tag: String,
    /// The amount of commits to Hyprland at buildtime
    pub commits: String,
    /// Aquamarine version
    #[serde(rename = "buildAquamarine")]
    pub build_aquamarine: String,
    /// The flags that Hyprland was built with
    pub flags: Vec<String>,
}
impl_on!(Version);

/// This struct holds information on the cursor position
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// The x position of the cursor
    pub x: i64,
    /// The y position of the cursor
    pub y: i64,
}
impl_on!(CursorPosition);

/// A keybinding returned from the binds command
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    /// Is it locked?
    pub locked: bool,
    /// Is it a mouse bind?
    pub mouse: bool,
    /// Does it execute on release?
    pub release: bool,
    /// Can it be held?
    pub repeat: bool,
    /// It's modmask
    pub modmask: u16,
    /// The submap its apart of
    pub submap: String,
    /// The key
    pub key: String,
    /// The keycode
    pub keycode: i16,
    /// The dispatcher to be executed
    pub dispatcher: String,
    /// The dispatcher arg
    pub arg: String,
    /// description from bind[d]
    pub description: String,
}

create_data_struct!(
    vector,
    name: Binds,
    command: DataCommands::Binds,
    holding_type: Bind,
    doc: "This struct holds a vector of binds"
);

/// Animation styles
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AnimationStyle {
    /// Slide animation
    Slide,
    /// Vertical slide animation
    SlideVert,
    /// Fading slide animation
    SlideFade,
    /// Fading slide animation in a vertical direction
    SlideFadeVert,
    /// Popin animation (with percentage)
    PopIn(u8),
    /// Fade animation
    Fade,
    /// Once animation used for gradient animation
    Once,
    /// Loop animation used for gradient animation
    Loop,
    /// No animation style
    None,
    /// Unknown style
    Unknown(String),
}

impl From<String> for AnimationStyle {
    fn from(value: String) -> Self {
        if value.starts_with("popin") {
            let mut iter = value.split(' ');
            iter.next();
            AnimationStyle::PopIn({
                let mut str = iter.next().unwrap_or("100%").to_string();
                str.remove(str.len() - 1);

                str.parse().unwrap_or(100_u8)
            })
        } else {
            match value.as_str() {
                "slide" => AnimationStyle::Slide,
                "slidevert" => AnimationStyle::SlideVert,
                "fade" => AnimationStyle::Fade,
                "slidefade" => AnimationStyle::SlideFade,
                "slidefadevert" => AnimationStyle::SlideFadeVert,
                "once" => AnimationStyle::Once,
                "loop" => AnimationStyle::Loop,
                "" => AnimationStyle::None,
                _ => AnimationStyle::Unknown(value),
            }
        }
    }
}
/// Bezier identifier
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BezierIdent {
    /// No bezier specified
    #[serde(rename = "")]
    None,
    /// The default bezier
    #[serde(rename = "default")]
    Default,
    /// A specified bezier
    #[serde(rename = "name")]
    Specified(String),
}

impl From<String> for BezierIdent {
    fn from(value: String) -> Self {
        match value.as_str() {
            "" => BezierIdent::None,
            "default" => BezierIdent::Default,
            _ => BezierIdent::Specified(value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct RawBezierIdent {
    pub name: String,
}

/// A bezier curve
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Bezier {
    ///. Name of the bezier
    pub name: String,
    /// X position of first point
    pub x0: f32,
    /// Y position of first point
    pub y0: f32,
    /// X position of second point
    pub x1: f32,
    /// Y position of second point
    pub y1: f32,
}

/// A struct representing a animation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct AnimationRaw {
    /// The name of the animation
    pub name: String,
    /// Is it overridden?
    pub overridden: bool,
    /// What bezier does it use?
    pub bezier: String,
    /// Is it enabled?
    pub enabled: bool,
    /// How fast is it?
    pub speed: f32,
    /// The style of animation
    pub style: String,
}

/// A struct representing a animation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Animation {
    /// The name of the animation
    pub name: String,
    /// Is it overridden?
    pub overridden: bool,
    /// What bezier does it use?
    pub bezier: BezierIdent,
    /// Is it enabled?
    pub enabled: bool,
    /// How fast is it?
    pub speed: f32,
    /// The style of animation
    pub style: AnimationStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct AnimationsRaw(Vec<AnimationRaw>, Vec<RawBezierIdent>);

/// Struct that holds animations and beziers
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Animations(pub Vec<Animation>, pub Vec<BezierIdent>);

impl HyprData for Animations {
    fn get() -> crate::Result<Self> {
        Self::instance_get(default_instance()?)
    }

    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?).await
    }

    fn instance_get(instance: &Instance) -> crate::Result<Self> {
        let out = instance.write_to_socket(command!(JSON, "{}", DataCommands::Animations))?;
        let des: AnimationsRaw = serde_json::from_str(&out)?;
        let AnimationsRaw(anims, beziers) = des;
        let new_anims: Vec<Animation> = anims
            .into_iter()
            .map(|item| Animation {
                name: item.name,
                overridden: item.overridden,
                bezier: item.bezier.into(),
                enabled: item.enabled,
                speed: item.speed,
                style: item.style.into(),
            })
            .collect();
        let new_bezs: Vec<BezierIdent> = beziers.into_iter().map(|item| item.name.into()).collect();
        Ok(Animations(new_anims, new_bezs))
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &Instance) -> crate::Result<Self> {
        let out = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::Animations))
            .await?;
        let des: AnimationsRaw = serde_json::from_str(&out)?;
        let AnimationsRaw(anims, beziers) = des;
        let new_anims: Vec<Animation> = anims
            .into_iter()
            .map(|item| Animation {
                name: item.name,
                overridden: item.overridden,
                bezier: item.bezier.into(),
                enabled: item.enabled,
                speed: item.speed,
                style: item.style.into(),
            })
            .collect();
        let new_bezs: Vec<BezierIdent> = beziers.into_iter().map(|item| item.name.into()).collect();
        Ok(Animations(new_anims, new_bezs))
    }
}

// HACK: shadow and decorate are actually missing from the hyprctl json output for some reason
// HACK: gaps_in and gaps_out are returned as arrays with 4 integers, even though Hyprland doesn't support per-side gaps
/// The rules of an individual workspace, as returned by hyprctl json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRuleset {
    /// The name of the workspace
    #[serde(rename = "workspaceString")]
    pub workspace_string: String,
    /// The monitor the workspace is on
    pub monitor: Option<String>,
    /// Is it default?
    pub default: Option<bool>,
    /// The gaps between windows
    #[serde(rename = "gapsIn")]
    pub gaps_in: Option<Vec<i64>>,
    /// The gaps between windows and monitor edges
    #[serde(rename = "gapsOut")]
    pub gaps_out: Option<Vec<i64>>,
    /// The size of window borders
    #[serde(rename = "borderSize")]
    pub border_size: Option<i64>,
    /// Are borders enabled?
    pub border: Option<bool>,
    /// Are shadows enabled?
    pub shadow: Option<bool>,
    /// Is rounding enabled?
    pub rounding: Option<bool>,
    /// Are window decorations enabled?
    pub decorate: Option<bool>,
    /// Is it persistent?
    pub persistent: Option<bool>,
}

create_data_struct!(
    vector,
    name: WorkspaceRules,
    command: DataCommands::WorkspaceRules,
    holding_type: WorkspaceRuleset,
    doc: "This struct holds a vector of workspace rules per workspace"
);
