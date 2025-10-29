use adw::gtk::gdk::Key;
use config_lib::Plugins;
use std::path::Path;
use tracing::debug_span;

mod actions;
mod applications;
#[cfg(feature = "calc")]
mod calc;
mod path;
mod search;
mod shell;
mod terminal;

pub use applications::get_stored_runs as get_applications_stored_runs;
pub use applications::reload_desktop_entries_map as reload_applications_desktop_entries_map;
use core_lib::transfer::{Identifier, PluginNames};

#[derive(Debug)]
pub struct SortableLaunchOption {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    /// Higher is better
    pub score: u64,
    pub grayed: bool,
    pub iden: Identifier,
    pub details_menu: Vec<DetailsMenuItem>,
}

#[derive(Debug)]
pub struct DetailsMenuItem {
    pub text: Box<str>,
    pub exec: Box<str>,
    pub iden: Identifier,
}

#[derive(Debug)]
pub struct StaticLaunchOption {
    pub text: Box<str>,
    pub details: Box<str>,
    pub icon: Option<Box<Path>>,
    pub key: char,
    pub iden: Identifier,
}

pub fn get_sortable_launch_options(
    plugins: &Plugins,
    text: &str,
    data_dir: &Path,
) -> Vec<SortableLaunchOption> {
    let mut matches = Vec::new();

    if let Some(config) = plugins.applications.as_ref() {
        debug_span!("applications").in_scope(|| {
            applications::get_sortable_options(
                &mut matches,
                text,
                config.run_cache_weeks,
                config.show_execs,
                config.show_actions_submenu,
                data_dir,
            );
        });
    }
    if plugins.calc.is_some() {
        #[cfg(feature = "calc")]
        debug_span!("calc").in_scope(|| {
            calc::get_calc_options(&mut matches, text);
        });
        #[cfg(not(feature = "calc"))]
        tracing::warn!("calc plugin is not enabled");
    }
    if plugins.path.is_some() {
        debug_span!("path").in_scope(|| path::get_path_options(&mut matches, text));
    }
    if let Some(config) = plugins.actions.as_ref() {
        debug_span!("actions")
            .in_scope(|| actions::get_actions_options(&mut matches, text, config));
    }

    // sort in reverse
    matches.sort_by(|a, b| b.score.cmp(&a.score));
    // trace!("matches: {:?}", matches);
    matches
}

pub fn get_static_launch_options(
    plugins: &Plugins,
    default_terminal: Option<&str>,
) -> Vec<StaticLaunchOption> {
    let mut matches = Vec::new();

    if plugins.shell.is_some() {
        debug_span!("shell").in_scope(|| {
            shell::get_static_options(&mut matches);
        });
    }
    if plugins.terminal.is_some() {
        debug_span!("terminal").in_scope(|| {
            terminal::get_static_options(&mut matches, default_terminal);
        });
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        debug_span!("search").in_scope(|| {
            search::get_static_options(&mut matches, &websearch.engines);
        });
    }

    matches
}

pub struct PluginReturn {
    pub show_animation: bool,
}

pub fn launch(
    iden: &Identifier,
    text: &str,
    default_terminal: Option<&str>,
    data_dir: &Path,
) -> PluginReturn {
    let _span = debug_span!("launch_plugin").entered();

    match iden.plugin {
        PluginNames::Applications => debug_span!("applications").in_scope(|| {
            applications::launch_option(
                iden.data.as_deref(),
                iden.data_additional.as_deref(),
                default_terminal,
                data_dir,
            )
        }),
        PluginNames::Shell => {
            debug_span!("shell").in_scope(|| shell::launch_option(text, default_terminal))
        }
        PluginNames::Terminal => {
            debug_span!("terminal").in_scope(|| terminal::launch_option(text, default_terminal))
        }
        PluginNames::WebSearch => {
            debug_span!("search").in_scope(|| search::launch_option(iden.data.as_deref(), text))
        }
        PluginNames::Path => debug_span!("path").in_scope(|| path::launch_option(text)),
        PluginNames::Calc => {
            #[cfg(feature = "calc")]
            debug_span!("calc").in_scope(|| calc::copy_result(iden.data.as_deref()));
            #[cfg(not(feature = "calc"))]
            tracing::warn!("calc plugin is not enabled");
            PluginReturn {
                show_animation: false,
            }
        }
        PluginNames::Actions => {
            debug_span!("actions").in_scope(|| actions::run_action(iden.data.as_deref()))
        }
    }
}

pub fn get_static_options_chars(plugins: &Plugins) -> Vec<Key> {
    let mut chars = Vec::new();

    if plugins.shell.is_some() {
        chars.append(&mut shell::get_chars());
    }
    if plugins.terminal.is_some() {
        chars.append(&mut terminal::get_chars());
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        chars.append(&mut search::get_chars(&websearch.engines));
    }

    chars
}
