//! # Keyword module
//!
//! This module is used for setting, and getting keywords
//!
//! ## Usage
//!
//! ```rust, no_run
//! use hyprland::keyword::Keyword;
//! fn main() -> hyprland::Result<()> {
//!     Keyword::get("some_keyword")?;
//!     Keyword::set("another_keyword", "the value to set it to")?;
//!
//!     Ok(())
//! }
//! ```

use crate::instance::Instance;
use crate::shared::*;
use crate::{default_instance, error::HyprError};
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// A Color made up of rgba values (0-255)
#[repr(Rust, packed)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HyprColor {
    /// Red Channel (0-255)
    pub r: u8,
    /// Green Channel (0-255)
    pub g: u8,
    /// Blue Channel (0-255)
    pub b: u8,
    /// Alpha (0-255)
    pub a: u8,
}

impl std::fmt::Display for HyprColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "rgba({:02x}{:02x}{:02x}{:02x})",
            self.r, self.g, self.b, self.a
        ))
    }
}

/// A Gradiant made up of HyprColor(s) and an angle
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HyprGradient {
    /// First gradiant color
    pub color0: HyprColor,
    /// Second gradiant color
    pub color1: Option<HyprColor>,
    /// Angle in degrees
    pub angle: u32,
}

impl std::fmt::Display for HyprGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (color1, space) = match &self.color1 {
            Some(s) => (s.to_string(), " "),
            None => (String::from(""), ""),
        };
        f.write_fmt(format_args!(
            "{}{}{} {}deg",
            self.color0, space, color1, self.angle
        ))
    }
}

/// Bounds used for custom Rects
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HyprRect {
    /// Bound Top
    pub top: i64,
    /// Bound Right
    pub right: i64,
    /// Bound Bottom
    pub bottom: i64,
    /// Bound Left
    pub left: i64,
}

impl std::fmt::Display for HyprRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0} {:0} {:0} {:0}",
            self.top, self.right, self.bottom, self.left
        ))
    }
}

impl TryFrom<&str> for HyprRect {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut s = s.trim().split(" ");
        let top = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let right = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let bottom = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let left = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        if s.next().is_some() {
            return Err(crate::error::HyprError::InvalidOptionValue);
        }

        Ok(Self {
            top,
            right,
            bottom,
            left,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Display)]
/// A parseable value type for custom options
pub enum Custom {
    /// Color Variant for Custom field
    HyprColor(HyprColor),
    /// Gradiant Variant for Custom field
    HyprGradient(HyprGradient),
    /// A general rect made of top, right, bottom, left
    HyprRect(HyprRect),
}

impl HyprColor {
    /// Convert an AARRGGBB u32 value to a HyprColor
    pub fn from_argb_u32(i: u32) -> Self {
        Self {
            a: (i >> 24) as u8,             // 0xFF000000
            r: ((i >> 16) & 0x00FF) as u8,  // 0x00FF0000
            g: ((i >> 8) & 0x0000FF) as u8, // 0x0000FF00
            b: (i & 0x000000FF) as u8,      // 0x000000FF
        }
    }

    /// Try to convert from &str in the argb legacy format to HyprColor,
    /// e.g. 0xeeb3ff1a
    pub fn try_from_argb_str(s: &str) -> Option<Self> {
        let s = s.trim().strip_prefix("0x").unwrap_or(s);
        u32::from_str_radix(s, 16).ok().map(Self::from_argb_u32)
    }

    /// Try to convert from &str in the rgba format to HyprColor,
    /// e.g. rgba(b3ff1aee), or the decimal equivalent rgba(179,255,26,0.933)
    pub fn try_from_rgba_str(s: &str) -> Option<Self> {
        s.trim()
            .strip_prefix("rgba(")?
            .strip_suffix(")")
            .and_then(|s| match s.contains(",") {
                // b10 parse (e.g., "rgba(255, 0, 170, 0.5)")
                true => {
                    let mut parts = s.split(",").enumerate().map(|(i, t)| {
                        if i < 3 {
                            t.trim().parse::<u8>().ok()
                        } else {
                            let f: f32 = t.trim().parse().ok()?;
                            Some((f.clamp(0.0, 1.0) * 255.0).round() as u8)
                        }
                    });

                    let r = parts.next()?? as u32;
                    let g = parts.next()?? as u32;
                    let b = parts.next()?? as u32;
                    let a = parts.next()?? as u32;
                    if parts.next().is_some() {
                        return None;
                    }

                    let u: u32 = (a << 24) | (r << 16) | (g << 8) | b;
                    Some(Self::from_argb_u32(u))
                }
                // b16 parse (e.g., "rgba(FF00AA7F)")
                false => {
                    let s = s.trim();
                    if s.len() != 6 {
                        return None;
                    }
                    let i = u32::from_str_radix(s, 16).ok()?;
                    let a = (i & 0xFF) << 24;
                    let i = (i >> 8) | a;
                    Some(Self::from_argb_u32(i))
                }
            })
    }

    /// Try to convert from &str in the rgb format to HyprColor,
    /// e.g. rgb(b3ff1a), or the decimal equivalent rgb(179,255,26)
    pub fn try_from_rgb_str(s: &str) -> Option<Self> {
        s.trim()
            .strip_prefix("rgb(")?
            .strip_suffix(")")
            .and_then(|s| match s.contains(",") {
                // b10 parse (e.g., "rgb(255, 0, 170)")
                true => {
                    let mut parts = s.split(",").map(|t| t.trim().parse::<u8>().ok());
                    let r = parts.next()?? as u32;
                    let g = parts.next()?? as u32;
                    let b = parts.next()?? as u32;
                    if parts.next().is_some() {
                        return None;
                    }
                    let a = 0xFFu32;

                    let u: u32 = (a << 24) | (r << 16) | (g << 8) | b;
                    Some(Self::from_argb_u32(u))
                }
                // b16 parse (e.g., "rgb(ff00aa)")
                false => {
                    let s = s.trim();
                    if s.len() != 6 {
                        return None;
                    }
                    let i = u32::from_str_radix(s, 16).ok()?;
                    let a = 0xFF000000;
                    let i = i | a;
                    Some(Self::from_argb_u32(i))
                }
            })
    }
}

impl TryFrom<&str> for HyprColor {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.trim().to_lowercase();
        if s.starts_with("rgba") {
            Self::try_from_rgba_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        } else if s.starts_with("rgb") {
            Self::try_from_rgb_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        } else {
            Self::try_from_argb_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        }
    }
}

impl TryFrom<&str> for HyprGradient {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut a = None;
        let s = s.trim().replace(", ", ",");
        let mut s = s.split(" ");
        let color0 = HyprColor::try_from(
            s.next()
                .ok_or(crate::HyprError::InvalidHyprGradiantFormat)?,
        )?;
        let mut color1 = None;
        let c1_angle = s
            .next()
            .ok_or(crate::HyprError::InvalidHyprGradiantFormat)?;
        if c1_angle.contains("deg") {
            let tmp = c1_angle
                .strip_suffix("deg")
                .ok_or(crate::HyprError::InvalidHyprGradiantFormat)?;
            a = Some(
                tmp.parse::<u32>()
                    .map_err(|_| crate::HyprError::InvalidHyprGradiantFormat)?,
            );
        } else {
            color1 = Some(HyprColor::try_from(c1_angle)?)
        }

        let angle = match a {
            Some(a) => a,
            None => {
                let c1_angle = s
                    .next()
                    .ok_or(crate::HyprError::InvalidHyprGradiantFormat)?;
                let tmp = c1_angle
                    .strip_suffix("deg")
                    .ok_or(crate::HyprError::InvalidHyprGradiantFormat)?;
                match tmp.parse::<u32>() {
                    Ok(i) => i,
                    Err(_) => return Err(crate::HyprError::InvalidHyprGradiantFormat),
                }
            }
        };

        if s.next().is_some() {
            return Err(crate::HyprError::InvalidHyprGradiantFormat);
        }

        Ok(HyprGradient {
            color0,
            color1,
            angle,
        })
    }
}

impl TryFrom<&str> for Custom {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c = s.trim().replace(", ", ",");
        let c = c.split(" ");
        let c = c.count();
        if c == 4 {
            Ok(Self::HyprRect(HyprRect::try_from(s)?))
        } else if c > 1 {
            Ok(Self::HyprGradient(HyprGradient::try_from(s)?))
        } else {
            Ok(Self::HyprColor(HyprColor::try_from(s)?))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct OptionRaw {
    pub option: String,
    pub set: bool,

    #[serde(flatten)]
    pub value: std::collections::HashMap<String, serde_json::Value>,

    #[serde(skip)]
    pub json: String,
}

/// This enum holds the possible values of a keyword/option
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OptionValue {
    /// A integer (64-bit)
    Int(i64),
    /// A floating point (64-point)
    Float(f64),
    /// A string
    String(String),
    /// A hyprland Color or Gradiant
    Custom(Custom),
    /// A Vector of 2 ints
    Vec2([i64; 2]),
    /// Could not parse value
    Unknown(String),
}

impl std::fmt::Display for OptionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionValue::Int(i) => f.write_fmt(format_args!("{:0}", i)),
            OptionValue::Float(fl) => f.write_fmt(format_args!("{:0}", fl)),
            OptionValue::String(s) => f.write_fmt(format_args!("{}", s)),
            OptionValue::Custom(custom) => f.write_fmt(format_args!("{}", custom)),
            OptionValue::Vec2(v) => f.write_fmt(format_args!("{} {}", v[0], v[1])),
            OptionValue::Unknown(s) => f.write_fmt(format_args!("Unknown Value type ({})", s)),
        }
    }
}

impl From<OptionValue> for String {
    fn from(opt: OptionValue) -> Self {
        opt.to_string()
    }
}

trait IsString {}
impl IsString for String {}
impl IsString for &str {}

impl<Str: ToString + IsString> From<Str> for OptionValue {
    fn from(str: Str) -> Self {
        if let Ok(c) = Custom::try_from(str.to_string().as_str()) {
            OptionValue::Custom(c)
        } else {
            OptionValue::String(str.to_string())
        }
    }
}

macro_rules! match_unknown {
    ($k:expr, $v:expr, $opt:ident) => {
        match $v {
            Some(vi) => OptionValue::$opt(vi),
            None => OptionValue::Unknown($k.to_string()),
        }
    };
}

impl TryFrom<&OptionRaw> for OptionValue {
    type Error = HyprError;
    fn try_from(raw: &OptionRaw) -> crate::Result<Self> {
        Ok(match raw.value.iter().next() {
            Some((k, v)) => match k.as_str() {
                "int" => match_unknown!(raw.json, v.as_i64(), Int),
                "float" => match_unknown!(raw.json, v.as_f64(), Float),
                "str" => match_unknown!(raw.json, v.as_str().map(|v| v.to_string()), String),
                "custom" => match_unknown!(
                    raw.json,
                    v.as_str().and_then(|v| Custom::try_from(v).ok()),
                    Custom
                ),
                "vec2" => {
                    if let Some(a) = v.as_array()
                        && a.len() == 2
                        && a[0].is_i64()
                        && a[1].is_i64()
                    {
                        return Ok(OptionValue::Vec2([
                            a[0].as_i64().ok_or(HyprError::InvalidOptionValue)?,
                            a[1].as_i64().ok_or(HyprError::InvalidOptionValue)?,
                        ]));
                    }
                    OptionValue::Unknown(raw.json.to_string())
                }
                _ => OptionValue::Unknown(raw.json.to_string()),
            },
            None => OptionValue::Unknown(raw.json.to_string()),
        })
    }
}

/// This struct holds a keyword
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyword {
    /// The identifier (or name) of the keyword
    pub option: String,
    /// The value of the keyword/option
    pub value: OptionValue,
    /// Is value overriden or not
    pub set: bool,
}

impl Keyword {
    /// This function sets a keyword's value
    pub fn set<Str: ToString, Opt: Into<OptionValue>>(key: Str, value: Opt) -> crate::Result<()> {
        Self::instance_set(default_instance()?, key, value)
    }

    /// This function sets a keyword's value
    pub fn instance_set<Str: ToString, Opt: Into<OptionValue>>(
        instance: &Instance,
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        let value = value.into();

        let value = match value {
            OptionValue::Unknown(_) => {
                return Err(crate::HyprError::InvalidOptionValue);
            }
            x => x,
        };

        // hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"), { locked = true })
        instance.write_to_socket(command!(
            Empty,
            "keyword {} {}",
            key.to_string(),
            value.to_string()
        ))?;
        Ok(())
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn set_async<Str: ToString, Opt: Into<OptionValue>>(
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        Self::instance_set_async(default_instance()?, key, value).await
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_set_async<Str: ToString, Opt: Into<OptionValue>>(
        instance: &Instance,
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(
                Empty,
                "keyword {} {}",
                key.to_string(),
                value.into().to_string()
            ))
            .await?;
        Ok(())
    }

    /// This function returns the value of a keyword
    pub fn get<Str: ToString>(key: Str) -> crate::Result<Self> {
        Self::instance_get(default_instance()?, key)
    }

    /// This function returns the value of a keyword
    pub fn instance_get<Str: ToString>(instance: &Instance, key: Str) -> crate::Result<Self> {
        let data = instance.write_to_socket(command!(JSON, "getoption {}", key.to_string()))?;
        if data == "no such option" {
            return Err(crate::error::HyprError::InvalidOptionKey(key.to_string()));
        }
        let mut deserialized: OptionRaw = serde_json::from_str(&data)?;
        deserialized.json = data;
        let value = OptionValue::try_from(&deserialized)?;

        let keyword = Keyword {
            option: deserialized.option,
            value,
            set: deserialized.set,
        };
        Ok(keyword)
    }

    /// This function returns the value of a keyword (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn get_async<Str: ToString>(key: Str) -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?, key).await
    }

    /// This function returns the value of a keyword (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_get_async<Str: ToString>(
        instance: &Instance,
        key: Str,
    ) -> crate::Result<Self> {
        let data = instance
            .write_to_socket_async(command!(JSON, "getoption {}", key.to_string()))
            .await?;
        if data == "no such option" {
            return Err(crate::error::HyprError::InvalidOptionKey(key.to_string()));
        }
        let mut deserialized: OptionRaw = serde_json::from_str(&data)?;
        deserialized.json = data;

        let value = OptionValue::try_from(&deserialized)?;

        let keyword = Keyword {
            option: deserialized.option,
            value,
            set: deserialized.set,
        };
        Ok(keyword)
    }
}
