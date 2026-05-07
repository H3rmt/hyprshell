use std::fmt;

pub fn format_string_field(key: &str, value: &str) -> String {
    format!("{key} = \"{value}\"")
}
pub fn format_bool_field(key: &str, value: bool) -> String {
    format!("{key} = {}", if value { "true" } else { "false" })
}
pub fn format_raw_field<T: fmt::Display>(key: &str, value: T) -> String {
    format!("{key} = {value}")
}
