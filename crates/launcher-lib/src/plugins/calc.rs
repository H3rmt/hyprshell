use crate::plugins::{Identifier, PluginNames, SortableLaunchOption};
use core_lib::WarnWithDetails;
use relm4::adw::gtk::gdk::Display;
use relm4::adw::gtk::prelude::DisplayExt;
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use tracing::{debug, trace};

fn get_context() -> Option<&'static RwLock<rink_core::Context>> {
    static MAP_LOCK: OnceLock<Option<RwLock<rink_core::Context>>> = OnceLock::new();
    MAP_LOCK
        .get_or_init(|| {
            rink_core::simple_context()
                .warn_details("unable to create calc context")
                .map(RwLock::new)
        })
        .as_ref()
}

pub fn init_context() {
    get_context();
}

pub fn get_calc_options(matches: &mut Vec<SortableLaunchOption>, text: &str) {
    if text.is_empty() {
        return;
    }

    let Some(context_lock) = get_context() else {
        return;
    };
    let Ok(mut context) = context_lock.write() else {
        return;
    };
    let eval = rink_core::one_line(&mut context, text);
    // let mut context: calc::Context<f64> = calc::Context::default();
    // let eval = context.evaluate(text);

    if let Ok(eval) = eval {
        trace!("Eval: {eval:?}");
        let (title, desc) = parse_result(eval);
        trace!("Added calc option: {title}, {desc:?}");
        matches.push(SortableLaunchOption {
            icon: Some(Box::from(Path::new("accessories-calculator"))),
            name: title.clone(),
            details: Box::from("Copy to clipboard"),
            details_long: desc,
            score: 0,
            grayed: false,
            iden: Identifier::data(PluginNames::Calc, title),
            details_menu: vec![],
        });
    } else {
        trace!("No option added: expression error: {eval:?}");
    }
}

pub fn copy_result(data: Option<&str>) -> bool {
    if let Some(data) = data
        && let Some(clipboard) = Display::default().map(|display| display.clipboard())
    {
        debug!("Copying result to clipboard: {}", data);
        clipboard.set_text(data.as_ref());
    }
    false
}

// remove `(....) 23.23`
// remove `approx. 34.34`
// remove `23/233, 0.09871244`
#[allow(clippy::map_unwrap_or)]
fn parse_result(result: String) -> (Box<str>, Option<Box<str>>) {
    if result.contains("approx. ") {
        return parse_result(result.replace("approx. ", ""));
    }
    if result.contains(", ") {
        return result
            .split_once(", ")
            .map(|(desc, title)| {
                let a = parse_result(title.to_string());
                let des = a.1.map(|s| format!("{s} ")).unwrap_or_default();
                (a.0, Some(Box::from(format!("{des}{desc}"))))
            })
            .unwrap_or_else(|| (result.into_boxed_str(), None));
    }
    if result.contains(" (") {
        return result
            .split_once(" (")
            .map(|(title, desc)| {
                (
                    Box::from(title),
                    Some(Box::from(desc.trim_end_matches(')'))),
                )
            })
            .unwrap_or_else(|| (result.into_boxed_str(), None));
    }

    (result.into_boxed_str(), None)
}

#[cfg(test)]
mod tests {
    use crate::plugins::calc::parse_result;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_approx_with_dimensions() {
        let result = parse_result("approx. 0.5217391 (dimensionless)".to_string());
        assert_eq!(result.0.as_ref(), "0.5217391");
        assert_eq!(result.1.as_deref(), Some("dimensionless"));
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_approx_with_dimensions_and_fraction() {
        let result = parse_result("12/23, approx. 0.5217391 (dimensionless)".to_string());
        assert_eq!(result.0.as_ref(), "0.5217391");
        assert_eq!(result.1.as_deref(), Some("dimensionless 12/23"));
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_approx() {
        let result = parse_result("approx. 42".to_string());
        assert_eq!(result.0.as_ref(), "42");
        assert_eq!(result.1, None);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_fraction() {
        let result = parse_result("1/2, 0.5".to_string());
        assert_eq!(result.0.as_ref(), "0.5");
        assert_eq!(result.1.as_deref(), Some("1/2"));
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_with_parentheses() {
        let result = parse_result("42 (answer)".to_string());
        assert_eq!(result.0.as_ref(), "42");
        assert_eq!(result.1.as_deref(), Some("answer"));
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_simple() {
        let result = parse_result("42".to_string());
        assert_eq!(result.0.as_ref(), "42");
        assert_eq!(result.1, None);
    }
}
