use crate::plugins::{Identifier, PluginNames, SortableLaunchOption};
use gtk::gdk::Display;
use gtk::prelude::DisplayExt;
use std::path::Path;
use tracing::{debug, trace};

pub(crate) fn get_calc_options(matches: &mut Vec<SortableLaunchOption>, text: &str) {
    if text.is_empty() {
        return;
    }
    let mut context: calc::Context<f64> = Default::default();
    let eval = context.evaluate(text);
    if let Ok(eval) = eval {
        matches.push(SortableLaunchOption {
            icon: Some(Box::from(Path::new("accessories-calculator"))),
            name: format!("{eval}").into_boxed_str(),
            details: Box::from(""),
            details_long: None,
            score: 0,
            iden: Identifier::data(PluginNames::Calc, format!("{eval}").into_boxed_str()),
            details_menu: vec![],
        });
        trace!("Added calc option: {eval}");
    } else {
        trace!("No option added: expression error: {eval:?}");
    }
}

pub fn copy_result(data: &Option<Box<str>>) -> bool {
    if let Some(data) = data {
        if let Some(clipboard) = Display::default().map(|display| display.clipboard()) {
            debug!("Copying result to clipboard: {}", data);
            clipboard.set_text(data.as_ref());
        }
    }
    false
}
