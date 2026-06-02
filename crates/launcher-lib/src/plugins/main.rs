use crate::plugin::{LaunchItem, MatchedLaunchItem, PluginItem, PluginReturn};
use crate::plugins::{actions, applications, path, search, shell, terminal};
use config_lib::Plugins;
use core_lib::transfer::{Identifier, PluginName};
use relm4::adw::gtk::gdk::Key;
use std::path::Path;
use tracing::debug_span;

pub fn init() {
    #[cfg(feature = "calc")]
    crate::plugins::calc::init_context();
}

/// Get launcher items that don't change depending on user input.
pub fn get_static_items(plugins: &Plugins, data_dir: &Path) -> Vec<LaunchItem> {
    let mut items = Vec::new();

    if let Some(config) = plugins.applications.as_ref() {
        debug_span!("applications").in_scope(|| {
            items.extend(applications::get_launch_items(
                config.run_cache_weeks,
                config.show_execs,
                config.show_actions_submenu,
                data_dir,
            ));
        });
    }
    if let Some(config) = plugins.actions.as_ref() {
        debug_span!("actions").in_scope(|| items.extend(actions::get_launch_items(config)));
    }

    items
}

pub fn get_static_plugins(plugins: &Plugins, default_terminal: Option<&str>) -> Vec<PluginItem> {
    let mut items = Vec::new();

    if plugins.shell.is_some() {
        debug_span!("shell").in_scope(|| items.extend(shell::get_static_items()));
    }
    if plugins.terminal.is_some() {
        debug_span!("terminal").in_scope(|| {
            items.extend(terminal::get_static_options(default_terminal));
        });
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        debug_span!("search")
            .in_scope(|| items.extend(search::get_static_options(&websearch.engines)));
    }

    items
}

pub fn get_input_driven_launch_items(plugins: &Plugins, text: &str) -> Vec<MatchedLaunchItem> {
    let mut out = Vec::new();

    if plugins.path.is_some() {
        debug_span!("path").in_scope(|| out.extend(path::get_launch_items(text)));
    }

    if plugins.calc.is_some() {
        #[cfg(feature = "calc")]
        debug_span!("calc").in_scope(|| {
            out.extend(crate::plugins::calc::get_launch_items(text));
        });
        #[cfg(not(feature = "calc"))]
        tracing::warn!("calc plugin is not enabled");
    }

    out.sort_by(|a, b| b.score.cmp(&a.score));
    out
}

pub fn launch(
    iden: &Identifier,
    text: &str,
    default_terminal: Option<&str>,
    data_dir: &Path,
    args: Option<&str>,
) -> PluginReturn {
    let _span = debug_span!("launch_plugin").entered();
    match iden.plugin {
        PluginName::Applications => debug_span!("applications").in_scope(|| {
            applications::launch_option(
                iden.data.as_deref(),
                iden.data_additional.as_deref(),
                default_terminal,
                data_dir,
            )
        }),
        PluginName::Shell => {
            debug_span!("shell").in_scope(|| shell::launch_option(text, default_terminal))
        }
        PluginName::Terminal => {
            debug_span!("terminal").in_scope(|| terminal::launch_option(text, default_terminal))
        }
        PluginName::WebSearch => {
            debug_span!("search").in_scope(|| search::launch_option(iden.data.as_deref(), text))
        }
        PluginName::Path => debug_span!("path").in_scope(|| path::launch_option(text)),
        PluginName::Calc => {
            #[cfg(feature = "calc")]
            debug_span!("calc")
                .in_scope(|| crate::plugins::calc::copy_result(iden.data.as_deref()));
            #[cfg(not(feature = "calc"))]
            tracing::warn!("calc plugin is not enabled");
            PluginReturn {
                show_animation: false,
            }
        }
        PluginName::Actions => debug_span!("actions").in_scope(|| {
            actions::run_action(
                iden.data.as_deref(),
                text,
                iden.data_additional.as_deref(),
                args,
            )
        }),
    }
}

pub fn get_static_options_chars(plugins: &Plugins) -> Vec<Key> {
    let mut chars = Vec::new();
    if plugins.shell.is_some() {
        chars.extend(shell::get_chars());
    }
    if plugins.terminal.is_some() {
        chars.extend(terminal::get_chars());
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        chars.extend(search::get_chars(&websearch.engines));
    }
    chars
}
