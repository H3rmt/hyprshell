use std::fmt::Display;

pub fn format_string<T: Display>(value: &T) -> String {
    format!("\"{}\"", value.to_string().escape_default())
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
