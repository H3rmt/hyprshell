use crate::plugins::{Identifier, PluginNames, PluginReturn, StaticLaunchOption};
use config_lib::SearchEngine;
use core_lib::WarnWithDetails;
use core_lib::default::get_default_desktop_file;
use exec_lib::run::run_program;
use exec_lib::switch::switch_client_by_initial_class;
use relm4::adw::gtk::gdk::Key;
use std::path::Path;
use tracing::{debug, trace, warn};

pub fn get_static_options(matches: &mut Vec<StaticLaunchOption>, config: &[SearchEngine]) {
    let browser = get_browser_info();
    let icon = browser.icon.clone();
    drop(browser);
    let mut count = 0;
    for engine in config {
        if engine.key.is_whitespace() {
            warn!("Plugin {} has no valid key set", engine.name);
        } else {
            matches.push(StaticLaunchOption {
                text: engine.name.clone(),
                details: format!("Search with {}", engine.name).into_boxed_str(),
                icon: icon.clone(),
                key: engine.key,
                iden: Identifier::data(PluginNames::WebSearch, engine.url.clone()),
            });
            count += 1;
        }
    }
    trace!("Added {count} static web search options");
}

pub fn launch_option(iden: Option<&str>, text: &str) -> PluginReturn {
    if text.is_empty() {
        debug!("No text to search for");
        return PluginReturn {
            show_animation: false,
        };
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
        run_program(&cmdline, None, false, None).warn_details("Failed to run program");

        // try to focus browser
        if let Some(class) = &browser.startup_wm_class {
            debug!("trying to focus browser with class: {}", class);
            switch_client_by_initial_class(class).warn_details("unable to focus browser");
        } else {
            trace!("not class to browser available");
        }
    }
    PluginReturn {
        show_animation: true,
    }
}

pub fn get_chars(config: &[SearchEngine]) -> Vec<Key> {
    config
        .iter()
        .filter_map(|engine| convert_to_key(engine.key))
        .collect()
}

pub struct BrowserData {
    pub exec: Box<str>,
    pub startup_wm_class: Option<Box<str>>,
    pub icon: Option<Box<Path>>,
}

pub(super) fn get_browser_info() -> BrowserData {
    let a = get_default_desktop_file("x-scheme-handler/https", |(entry, ini)| {
        if let Some(section) = ini.get_section("Desktop Entry") {
            let exec = section.get_first("Exec");
            let startup_wm_class = section.get_first("StartupWMClass");
            let icon = section.get_first_as_path("Icon");
            if let Some(exec) = exec {
                trace!(
                    "Found default browser file: {} with exec: {exec}, icon: {icon:?} and startup_wm_class: {startup_wm_class:?}",
                    entry.path().display()
                );
                return Some(BrowserData {
                    exec,
                    startup_wm_class,
                    icon,
                });
            }
        }
        None
    });
    a.unwrap_or_else(|| {
        warn!("No default browser found! (using firefox and gdbus to open)");
        BrowserData {
            exec: Box::from(
                r#"gdbus call --session --dest="org.freedesktop.portal.Desktop" --object-path=/org/freedesktop/portal/desktop --method=org.freedesktop.portal.OpenURI.OpenURI '' '%u' '{}'"#,
            ),
            startup_wm_class: Some(Box::from("firefox")),
            icon: Some(Box::from(Path::new("firefox"))),
        }
    })
}

pub const fn convert_to_key(char: char) -> Option<Key> {
    match char {
        'a' => Some(Key::a),
        'b' => Some(Key::b),
        'c' => Some(Key::c),
        'd' => Some(Key::d),
        'e' => Some(Key::e),
        'f' => Some(Key::f),
        'g' => Some(Key::g),
        'h' => Some(Key::h),
        'i' => Some(Key::i),
        'j' => Some(Key::j),
        'k' => Some(Key::k),
        'l' => Some(Key::l),
        'm' => Some(Key::m),
        'n' => Some(Key::n),
        'o' => Some(Key::o),
        'p' => Some(Key::p),
        'q' => Some(Key::q),
        'r' => Some(Key::r),
        's' => Some(Key::s),
        't' => Some(Key::t),
        'u' => Some(Key::u),
        'v' => Some(Key::v),
        'w' => Some(Key::w),
        'x' => Some(Key::x),
        'y' => Some(Key::y),
        'z' => Some(Key::z),
        _ => None,
    }
}
