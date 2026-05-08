use crate::dispatch_new::{Direction, Dispatch, ToDispatch, WindowIdentifier};
use crate::instance::Instance;
use crate::lua::{format_bool_field, format_string_field};
use crate::{command, default_instance};
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
pub struct Binding<D: ToDispatch> {
    /// All the mods
    pub mods: Vec<Mod>,
    /// The key
    pub key: String,
    /// Dispatcher
    pub dispatcher: D,
    /// Bind flags
    pub flags: Vec<Flag>,
}
impl<D: ToDispatch> fmt::Display for Binding<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hl.bind(")?;
        let mut bind = String::new();
        for m in &self.mods {
            bind.push_str(&format!("{m} + "));
        }
        bind.push_str(&self.key);
        f.write_str(&format!("\"{}\"", bind))?;
        f.write_str(", ")?;

        f.write_str(&self.dispatcher.to_string())?;

        f.write_str(", { ")?;

        for effect in &self.flags {
            effect.fmt(f)?;
        }

        f.write_str("})")
    }
}

impl<D: ToDispatch> Binding<D> {
    /// Binds a keybinding
    pub fn bind(&self) -> crate::Result<()> {
        self.instance_bind(default_instance()?)
    }
    /// Binds a keybinding
    pub fn instance_bind(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        let ret = instance.write_to_socket(command!(Empty, "eval {}", lua))?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not bind key: {}",
                ret
            )));
        }
        Ok(())
    }

    /// Binds a keybinding (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn bind_async(&self) -> crate::Result<()> {
        self.instance_bind_async(default_instance()?).await
    }

    /// Binds a keybinding (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_bind_async(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.to_string();
        let ret = instance
            .write_to_socket_async(command!(Empty, "eval {}", lua))
            .await?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not bind key: {}",
                ret
            )));
        }
        Ok(())
    }
}

#[test]
fn test_key_bindinds() {
    let binds = vec![
        (
            Binding {
                mods: vec![],
                key: "TAB".to_string(),
                dispatcher: Dispatch::FocusDirection(Direction::Up),
                flags: vec![],
            },
            r#"hl.bind("TAB"), hl.dsp.focus({ direction = "u", }), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Super, Mod::Shift],
                key: "1".to_string(),
                dispatcher: Dispatch::ExecCmd("ls -la".into(), None),
                flags: vec![Flag::Description(r#"move"a""#.into()), Flag::AutoConsuming],
            },
            r#"hl.bind("SUPER + SHIFT + 1"), hl.dsp.exec_cmd("ls -la"), { description = "move\"a\"", auto_consuming = true, })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::FocusLast,
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.focus({ last }), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::Exit,
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.exit(), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::SubMap("submap".into()),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.submap("submap"), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::Pass(None),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.pass({ }), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::Pass(Some(WindowIdentifier::InitialTitleRegularExpression(
                    "Proton P.*\\".to_string(),
                ))),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.pass({ window = "initialtitle:Proton P.*\\", }), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::Event("test".into()),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.event("test"), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::ForceIdle(232323),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.force_idle(232323), { })"#,
        ),
        (
            Binding {
                mods: vec![Mod::Alt],
                key: "tab".to_string(),
                dispatcher: Dispatch::NoOp(),
                flags: vec![],
            },
            r#"hl.bind("ALT + tab"), hl.dsp.no_op(), { })"#,
        ),
    ];
    for (bind, lua) in binds {
        assert_eq!(bind.to_string(), lua);
    }
}
