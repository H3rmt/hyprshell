use crate::lua::{format_bool_field, format_string_field};
use derive_more::Display;
use std::fmt;
use std::fmt::Pointer;

/// Enum for mod keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Mod {
    #[display("SUPER")]
    Super,
    #[display("SHIFT")]
    Shift,
    #[display("ALT")]
    Alt,
    #[display("CTRL")]
    Ctrl,
}

/// Enum for bind flags
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Flag {
    #[display("{}", format_bool_field("locked", true))]
    Locked,
    #[display("{}", format_bool_field("release", true))]
    Release,
    #[display("{}", format_bool_field("click", true))]
    Click,
    #[display("{}", format_bool_field("drag", true))]
    Drag,
    #[display("{}", format_bool_field("long_press", true))]
    LongPress,
    #[display("{}", format_bool_field("repeating", true))]
    Repeating,
    #[display("{}", format_bool_field("non_consuming", true))]
    NonConsuming,
    #[display("{}", format_bool_field("auto_consuming", true))]
    AutoConsuming,
    #[display("{}", format_bool_field("mouse", true))]
    Mouse,
    #[display("{}", format_bool_field("transparent", true))]
    Transparent,
    #[display("{}", format_bool_field("ignore_mods", true))]
    IgnoreMods,
    #[display("{}", format_bool_field("separate", true))]
    Separate,
    #[display("{}", format_string_field("description", _0))]
    Description(String),
    #[display("{}", format_bool_field("bypass", true))]
    Bypass,
    #[display("{}", format_bool_field("submap_universal", true))]
    SubmapUniversal,
    // TODO
    // Devices,
}

/// A struct providing a key bind
#[derive(Debug, Clone)]
pub struct Binding {
    /// All the mods
    pub mods: Vec<Mod>,
    /// The key
    pub key: String,
    /// Bind flags
    pub flags: Vec<Flag>,
}
impl fmt::Display for Binding {
    // hl.bind(cfg.mainmod .. " + F", hl.dsp.window.fullscreen())
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hl.bind(")?;
        let mut bind = String::new();
        for m in &self.mods {
            bind.push_str(&format!("{m} + "));
        }
        bind.push_str(&self.key);
        f.write_str(&format!("\"{}\"", bind))?;
        f.write_str("), ")?;

        // TODO bind
        f.write_str("nix")?;

        f.write_str(", {")?;

        for effect in &self.flags {
            effect.fmt(f)?;
            f.write_str(", ")?;
        }

        f.write_str("})")
    }
}

#[test]
fn test_key_bindinds() {
    let rules = vec![
        (
            Binding {
                mods: vec![],
                key: "TAB".to_string(),
                flags: vec![],
            },
            r#"hl.bind("TAB"), nix, {})"#,
        ),
        (
            Binding {
                mods: vec![Mod::Super, Mod::Shift],
                key: "1".to_string(),
                flags: vec![Flag::Description(r#"move\a"#.into()), Flag::AutoConsuming],
            },
            r#"hl.bind("SUPER + SHIFT + 1"), nix, {description = "move", auto_consuming = true, })"#,
        ),
    ];
    for (rule, lua) in rules {
        assert_eq!(rule.to_string(), lua);
    }
}
