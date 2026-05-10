use crate::instance::Instance;
use crate::lua::{format_bool_field, format_raw_field, format_string_field};
use crate::{command, default_instance};
use derive_more::Display;
use std::fmt;
use std::fmt::Write;

/// This struct holds a keyword
#[derive(Debug, Clone)]
pub struct WindowRule {
    /// The name of the rule
    pub name: Option<String>,
    /// The match options
    pub r#match: Vec<WindowMatch>,
    /// The effects
    pub effects: Vec<WindowEffect>,
}
impl fmt::Display for WindowRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hl.window_rule({")?;

        if let Some(name) = &self.name {
            f.write_str(&format_string_field("name", name))?;
        }

        f.write_str("match = {")?;

        for mat in &self.r#match {
            mat.fmt(f)?;
        }

        f.write_str("}, ")?;

        for effect in &self.effects {
            effect.fmt(f)?;
        }

        f.write_str("})")
    }
}
impl WindowRule {
    /// This function sets a keyword's value
    pub fn apply(&self) -> crate::Result<()> {
        self.instance_apply(default_instance()?)
    }

    /// This function sets a keyword's value
    pub fn instance_apply(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        let ret = instance.write_to_socket(command!(Empty, "eval {}", lua))?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not apply rule: {}",
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
            .write_to_socket_async(command!(Empty, "eval {}", lua))
            .await?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not apply rule: {}",
                ret
            )));
        }
        Ok(())
    }
}

/// Enum containing all match options
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum WindowMatch {
    #[display("{}", format_string_field("class", _0))]
    Class(String),
    #[display("{}", format_string_field("title", _0))]
    Title(String),
    #[display("{}", format_string_field("initial_class", _0))]
    InitialClass(String),
    #[display("{}", format_string_field("initial_title", _0))]
    InitialTitle(String),
    #[display("{}", format_string_field("tag", _0))]
    Tag(String),
    #[display("{}", format_bool_field("xwayland", *_0))]
    XWayland(bool),
    #[display("{}", format_bool_field("float", *_0))]
    Float(bool),
    #[display("{}", format_bool_field("fullscreen", *_0))]
    Fullscreen(bool),
    #[display("{}", format_bool_field("pin", *_0))]
    Pin(bool),
    #[display("{}", format_bool_field("focus", *_0))]
    Focus(bool),
    #[display("{}", format_bool_field("group",* _0))]
    Group(bool),
    #[display("{}", format_bool_field("modal", *_0))]
    Modal(bool),
    #[display("{}", format_raw_field("fullscreen_state_client", _0))]
    FullscreenStateClient(u8),
    #[display("{}", format_raw_field("fullscreen_state_internal", _0))]
    FullscreenStateInternal(u8),
    #[display("{}", format_string_field("workspace", _0))]
    Workspace(String),
    #[display("{}", format_string_field("content", _0))]
    Content(String),
    #[display("{}", format_string_field("xdg_tag", _0))]
    XdgTag(String),
}

/// Enum containing all effects
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum WindowEffect {
    #[display("{}", format_bool_field("float", *_0))]
    Float(bool),
    #[display("{}", format_bool_field("title", *_0))]
    Tile(bool),
    #[display("{}", format_bool_field("fullscreen", *_0))]
    Fullscreen(bool),
    #[display("{}", format_bool_field("maximize", *_0))]
    Maximize(bool),
    #[display("{}", format_string_field("fullscreen_state", _0))]
    FullscreenState(String),
    #[display("{}", format_string_field("move", _0))]
    Move(String),
    #[display("{}", format_string_field("size", _0))]
    Size(String),
    #[display("{}", format_bool_field("center", *_0))]
    Center(bool),
    #[display("{}", format_bool_field("pseudo", *_0))]
    Pseudo(bool),
    #[display("{}", format_string_field("monitor", _0))]
    Monitor(String),
    #[display("{}", format_string_field("workspace", _0))]
    Workspace(String),
    #[display("{}", format_bool_field("no_initial_focus", *_0))]
    NoInitialFocus(bool),
    #[display("{}", format_bool_field("pin", *_0))]
    Pin(bool),
    #[display("{}", format_string_field("group", _0))]
    Group(String),
    #[display("{}", format_string_field("suppress_event", _0))]
    SuppressEvent(String),
    #[display("{}", format_string_field("content", _0))]
    Content(String),
    #[display("{}", format_raw_field("no_close_for", _0))]
    NoCloseFor(i64),
    #[display("{}", format_raw_field("scrolling_width", _0))]
    ScrollingWidth(i64),
}

#[test]
fn test_window_rules() {
    let rules = vec![
        (
            WindowRule {
                name: Some("apply-something".into()),
                r#match: vec![WindowMatch::Class("my-class".into())],
                effects: vec![WindowEffect::Center(false), WindowEffect::Float(true)],
            },
            r#"hl.window_rule({name = "apply-something", match = {class = "my-class", }, center = false, float = true, })"#,
        ),
        (
            WindowRule {
                name: None,
                r#match: vec![WindowMatch::Focus(false), WindowMatch::Tag("my-tag".into())],
                effects: vec![
                    WindowEffect::NoCloseFor(230434),
                    WindowEffect::FullscreenState("1 2".into()),
                    WindowEffect::Content("video".into()),
                ],
            },
            r#"hl.window_rule({match = {focus = false, tag = "my-tag", }, no_close_for = 230434, fullscreen_state = "1 2", content = "video", })"#,
        ),
        (
            WindowRule {
                name: Some("some-layer-rule".into()),
                r#match: vec![
                    WindowMatch::InitialClass("test".into()),
                    WindowMatch::XWayland(false),
                    WindowMatch::Content("video".into()),
                ],
                effects: vec![
                    WindowEffect::Monitor("1".into()),
                    WindowEffect::Workspace("2".into()),
                    WindowEffect::Group("3".into()),
                    WindowEffect::Pin(true),
                    WindowEffect::NoInitialFocus(true),
                    WindowEffect::ScrollingWidth(234234),
                ],
            },
            r#"hl.window_rule({name = "some-layer-rule", match = {initial_class = "test", xwayland = false, content = "video", }, monitor = "1", workspace = "2", group = "3", pin = true, no_initial_focus = true, scrolling_width = 234234, })"#,
        ),
    ];
    for (rule, lua) in rules {
        assert_eq!(rule.to_string(), lua);
    }
}

/// This struct holds a keyword
#[derive(Debug, Clone)]
pub struct LayerRule {
    /// The name of the rule
    pub name: Option<String>,
    /// The match options
    pub r#match: Vec<LayerMatch>,
    /// The effects
    pub effects: Vec<LayerEffect>,
}
impl fmt::Display for LayerRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hl.layer_rule({")?;

        if let Some(name) = &self.name {
            f.write_str(&format_string_field("name", name))?;
        }

        f.write_str("match = {")?;

        for mat in &self.r#match {
            mat.fmt(f)?;
        }

        f.write_str("}, ")?;

        for effect in &self.effects {
            effect.fmt(f)?;
        }

        f.write_str("})")
    }
}
impl LayerRule {
    /// This function sets a keyword's value
    pub fn apply(&self) -> crate::Result<()> {
        self.instance_apply(default_instance()?)
    }

    /// This function sets a keyword's value
    pub fn instance_apply(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        instance.write_to_socket(command!(Empty, "eval {}", lua))?;
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
        instance
            .write_to_socket_async(command!(Empty, "eval {}", lua))
            .await?;
        Ok(())
    }
}

/// Enum containing all match options
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum LayerMatch {
    #[display("{}", format_string_field("namespace", _0))]
    Namespace(String),
}

/// Enum containing all effects
#[derive(Debug, Clone, PartialEq, Display)]
pub enum LayerEffect {
    #[display("{}", format_bool_field("no_anim", *_0))]
    NoAnim(bool),
    #[display("{}", format_bool_field("blur", *_0))]
    Blur(bool),
    #[display("{}", format_bool_field("blur_popups", *_0))]
    BlurPopups(bool),
    #[display("{}", format_raw_field("ignore_alpha", _0))]
    IgnoreAlpha(f32),
    #[display("{}", format_bool_field("dim_around", *_0))]
    DimAround(bool),
    #[display("{}", format_bool_field("xray", *_0))]
    Xray(bool),
    #[display("{}", format_string_field("animation", _0))]
    Animation(String),
    #[display("{}", format_raw_field("order", _0))]
    Order(i64),
    #[display("{}", format_raw_field("above_lock", _0))]
    AboveLock(i64),
    #[display("{}", format_bool_field("no_screen_share", *_0))]
    NoScreenShare(bool),
}

#[test]
fn test_layer_rules() {
    let rules = vec![
        (
            LayerRule {
                name: Some("apply-something".into()),
                r#match: vec![LayerMatch::Namespace("my-layer".into())],
                effects: vec![
                    LayerEffect::Animation("test".into()),
                    LayerEffect::Order(-20),
                ],
            },
            r#"hl.layer_rule({name = "apply-something", match = {namespace = "my-layer", }, animation = "test", order = -20, })"#,
        ),
        (
            LayerRule {
                name: None,
                r#match: vec![LayerMatch::Namespace("my-layer".into())],
                effects: vec![LayerEffect::Xray(false), LayerEffect::DimAround(true)],
            },
            r#"hl.layer_rule({match = {namespace = "my-layer", }, xray = false, dim_around = true, })"#,
        ),
        (
            LayerRule {
                name: Some("some-layer-rule".into()),
                r#match: vec![LayerMatch::Namespace("my-layer-2".into())],
                effects: vec![
                    LayerEffect::IgnoreAlpha(0.53),
                    LayerEffect::Blur(false),
                    LayerEffect::AboveLock(3334),
                ],
            },
            r#"hl.layer_rule({name = "some-layer-rule", match = {namespace = "my-layer-2", }, ignore_alpha = 0.53, blur = false, above_lock = 3334, })"#,
        ),
    ];
    for (rule, lua) in rules {
        assert_eq!(rule.to_string(), lua);
    }
}
