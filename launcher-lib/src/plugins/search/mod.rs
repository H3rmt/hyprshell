mod default;

use crate::plugins::search::default::get_browser_info;
use crate::plugins::{Identifier, PluginNames, StaticLaunchOption};
use crate::util::convert_to_key;
use config_lib::SearchEngine;
use core_lib::WarnWithDetails;
pub use default::reload_default_browser;
use exec_lib::run::run_program;
use exec_lib::switch::switch_client_by_initial_class;
use gtk::gdk::Key;
use tracing::{debug, trace, warn};

pub fn get_static_options(matches: &mut Vec<StaticLaunchOption>, config: &[SearchEngine]) {
    let browser = get_browser_info();
    let icon = browser.icon.clone();
    drop(browser);
    for engine in config.iter() {
        if !engine.key.is_whitespace() {
            matches.push(StaticLaunchOption {
                text: engine.name.clone(),
                details: format!("Search with {}", engine.name).into_boxed_str(),
                icon: icon.clone(),
                key: engine.key,
                iden: Identifier::data(PluginNames::WebSearch, engine.url.clone()),
            });
        } else {
            warn!("Plugin {} has no valid key set", engine.name);
        }
    }
}

pub fn launch_option(iden: &Option<Box<str>>, text: &str) -> bool {
    if text.is_empty() {
        debug!("No text to search for");
        return false;
    }
    if let Some(iden) = iden {
        let url = iden.replace("{}", text);
        debug!("Launching URL: {}", url);
        let browser = get_browser_info();
        let cmdline = if ["%u", "%U", "%f", "%F"]
            .iter()
            .any(|repl| browser.exec.contains(repl))
        {
            let mut exec = browser.exec.to_string();
            for repl in ["%u", "%U", "%f", "%F"] {
                exec = exec.replace(repl, &format!("'{url}'"));
            }
            exec
        } else {
            format!("{} '{}'", browser.exec, url)
        };
        debug!("Launching browser: {}", cmdline);
        run_program(&cmdline, None, false, &None).warn_details("Failed to run program");

        // try to focus browser
        if let Some(class) = &browser.startup_wm_class {
            debug!("trying to focus browser with class: {}", class);
            switch_client_by_initial_class(class).warn_details("unable to focus browser");
        } else {
            trace!("not class to browser available")
        }
    }
    true
}

pub(crate) fn get_chars(config: &[SearchEngine]) -> Vec<Key> {
    config
        .iter()
        .flat_map(|engine| convert_to_key(engine.key))
        .collect()
}
