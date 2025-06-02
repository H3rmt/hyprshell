use core_lib::config::Plugins;
use std::path::Path;
use tracing::{Level, span};

mod applications;
mod search;
mod shell;
mod terminal;

#[cfg(feature = "calc")]
mod calc;

pub use applications::get_stored_runs as get_applications_stored_runs;
pub use applications::reload_desktop_map as reload_applications_desktop_map;
use core_lib::transfer::{Identifier, PluginNames};
pub use search::reload_default_browser as reload_search_default_browser;

#[derive(Debug)]
pub struct SortableLaunchOption {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub score: u64,
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

pub fn iden_to_str(iden: &Identifier) -> String {
    format!(
        "{}:{}",
        iden.plugin as u8,
        iden.identifier.as_deref().unwrap_or_default()
    )
}

pub fn get_sortable_launch_options(
    plugins: &Plugins,
    text: &str,
    data_dir: &Path,
) -> Vec<SortableLaunchOption> {
    let mut matches = Vec::new();

    if let Some(config) = plugins.applications.as_ref() {
        applications::get_sortable_options(
            &mut matches,
            text,
            config.run_cache_weeks,
            config.show_execs,
            data_dir,
        );
    }
    if plugins.calc.is_some() {
        #[cfg(feature = "calc")]
        calc::get_calc_options(&mut matches, text);
        #[cfg(not(feature = "calc"))]
        tracing::warn!("calc plugin is not enabled");
    }

    // sort in reverse
    matches.sort_by(|a, b| b.score.cmp(&a.score));
    // trace!("matches: {:?}", matches);
    matches
}

pub fn get_static_launch_options(
    plugins: &Plugins,
    default_terminal: &Option<Box<str>>,
) -> Vec<StaticLaunchOption> {
    let mut matches = Vec::new();

    if plugins.shell.is_some() {
        shell::get_static_options(&mut matches);
    }
    if plugins.terminal.is_some() {
        terminal::get_static_options(&mut matches, default_terminal);
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        search::get_static_options(&mut matches, &websearch.engines);
    }

    matches
}

pub fn launch(
    iden: &Identifier,
    text: &str,
    default_terminal: &Option<Box<str>>,
    data_dir: &Path,
) -> bool {
    let _span = span!(Level::TRACE, "launch_plugin").entered();

    match iden.plugin {
        PluginNames::Applications => {
            applications::launch_option(&iden.identifier, default_terminal, data_dir)
        }
        PluginNames::Shell => shell::launch_option(text, default_terminal),
        PluginNames::Terminal => terminal::launch_option(text, default_terminal),
        PluginNames::WebSearch => search::launch_option(&iden.identifier, text),
        PluginNames::Calc => {
            #[cfg(feature = "calc")]
            calc::copy_result(&iden.identifier);
            #[cfg(not(feature = "calc"))]
            tracing::warn!("calc plugin is not enabled");
            false
        }
    }
}

pub fn get_static_options_chars(plugins: &Plugins) -> Vec<char> {
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
