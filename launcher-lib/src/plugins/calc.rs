use crate::plugins::{Identifier, PluginNames, SortableLaunchOption};
use gtk::prelude::DisplayExt;
use std::path::Path;
use tracing::debug;

pub(crate) fn get_calc_options(matches: &mut Vec<SortableLaunchOption>, text: &str) {
    let mut context: calc::Context<f64> = Default::default();
    let eval = context.evaluate(text);
    if let Ok(eval) = eval {
        matches.push(SortableLaunchOption {
            icon: Some(Box::from(Path::new("accessories-calculator"))),
            name: format!("{eval}").into_boxed_str(),
            details: Box::from(""),
            details_long: None,
            score: 0,
            iden: Identifier {
                plugin: PluginNames::Calc,
                identifier: Some(format!("{eval}").into_boxed_str()),
            },
            details_menu: vec![],
        });
    } else {
        // trace!("expression error: {eval:?}");
    }
}

pub fn copy_result(iden: &Option<Box<str>>) -> bool {
    if let Some(iden) = iden {
        if let Some(clipboard) = gtk::gdk::Display::default().map(|display| display.clipboard()) {
            debug!("Copying result to clipboard: {}", iden);
            clipboard.set_text(iden.as_ref());
        }
    }
    false
}
