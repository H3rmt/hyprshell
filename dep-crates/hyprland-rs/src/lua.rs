use crate::dispatch_new::Dispatch;
use crate::instance::Instance;
use crate::{command, default_instance};
use std::fmt::Display;

pub fn format_string<T: Display>(value: &T) -> String {
    // return format!("\"{}\"", value.to_string().escape_default());
    let mut f = String::with_capacity(value.to_string().len() + 2);
    f.push('"');
    for ch in value.to_string().chars() {
        match ch {
            '\\' => f.push_str("\\\\"),
            '"' => f.push_str("\\\""),
            '\n' => f.push_str("\\n"),
            '\r' => f.push_str("\\r"),
            '\t' => f.push_str("\\t"),
            c => f.push(c),
        }
    }
    f.push('"');
    f
}
pub fn format_string_opt<T: Display>(value: &Option<T>) -> String {
    match value {
        Some(v) => format_string(v),
        None => String::new(),
    }
}

pub fn format_string_field<T: Display>(key: &str, value: T) -> String {
    format!("{key} = {}, ", format_string(&value))
}
pub fn format_string_field_opt<T: Display>(key: &str, value: &Option<T>) -> String {
    match value {
        Some(v) => format_string_field(key, v),
        None => String::new(),
    }
}

pub fn format_bool_field(key: &str, value: bool) -> String {
    format!("{key} = {}, ", if value { "true" } else { "false" })
}
pub fn format_bool_field_opt(key: &str, value: &Option<bool>) -> String {
    match value {
        Some(v) => format_bool_field(key, *v),
        None => String::new(),
    }
}

pub fn format_raw_field<T: Display>(key: &str, value: T) -> String {
    format!("{key} = {value}, ")
}
pub fn format_raw_field_opt<T: Display>(key: &str, value: &Option<T>) -> String {
    match value {
        Some(v) => format_raw_field(key, v),
        None => String::new(),
    }
}

#[test]
fn test_format_string() {
    assert_eq!(format_string(&r#" test "#), r#"" test ""#);
    assert_eq!(format_string(&r#" "test" "#), r#"" \"test\" ""#);
    assert_eq!(format_string(&r#" "test\n" "#), r#"" \"test\\n\" ""#);
    assert_eq!(format_string(&r#" "test\r" "#), r#"" \"test\\r\" ""#);
    assert_eq!(format_string(&r#" "test\t" "#), r#"" \"test\\t\" ""#);
    assert_eq!(format_string(&r#" test\\ "#), r#"" test\\\\ ""#);
    assert_eq!(format_string(&r#" "test\"" "#), r#"" \"test\\\"\" ""#);
    assert_eq!(
        format_string(&r#" "test\n\r\t\\" "#),
        r#"" \"test\\n\\r\\t\\\\\" ""#
    );
    assert_eq!(format_string(&r#" test's "#), r#"" test's ""#);
}

#[derive(Debug, Clone)]
pub struct EvalRaw(String);

impl EvalRaw {
    pub fn new(lua: impl Display) -> Self {
        Self(lua.to_string())
    }

    /// This function sets a keyword's value
    pub fn eval(&self) -> crate::Result<()> {
        self.instance_eval(default_instance()?)
    }

    /// This function sets a keyword's value
    pub fn instance_eval(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.0.to_string();
        let ret = instance.write_to_socket(command!(Empty, "eval {}", lua))?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not eval: {}",
                ret
            )));
        }
        Ok(())
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn eval_async(&self) -> crate::Result<()> {
        self.instance_eval_async(default_instance()?).await
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_eval_async(&self, instance: &Instance) -> crate::Result<()> {
        let lua = self.0.to_string();
        let ret = instance
            .write_to_socket_async(command!(Empty, "eval {}", lua))
            .await?;
        if ret != "ok" {
            return Err(crate::error::HyprError::NotOkDispatch(format!(
                "Could not eval: {}",
                ret
            )));
        }
        Ok(())
    }
}
