use crate::plugins::{actions, applications, path, search, shell, terminal};
use config_lib::Plugins;
use core_lib::transfer::{Identifier, PluginName};
use nucleo::pattern::Pattern;
use relm4::adw::gtk::gdk::Key;
use std::path::Path;
use tracing::{debug_span, trace};

#[derive(Debug)]
pub struct SortableLaunchOption {
    pub names: Box<[Box<str>]>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub bonus_score: u64,
    // if this is true, only direct match is allowed
    pub takes_args: bool,
    pub enabled: bool,
    pub iden: Identifier,
    pub subactions: Vec<SortableLaunchOption>,
}

#[derive(Debug)]
pub struct SortedLaunchOption {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub takes_args: bool,
    pub enabled: bool,
    pub iden: Identifier,
    pub subactions: Vec<SortableLaunchOption>,
}

#[derive(Debug)]
pub struct StaticLaunchOption {
    pub text: Box<str>,
    pub details: Box<str>,
    pub icon: Option<Box<Path>>,
    pub key: char,
    pub iden: Identifier,
    pub enabled: bool,
}

pub fn get_sorted_launch_options(
    plugins: &Plugins,
    text: &str,
    data_dir: &Path,
) -> Vec<(u64, SortedLaunchOption)> {
    let mut matches = Vec::new();

    if let Some(config) = plugins.applications.as_ref() {
        debug_span!("applications").in_scope(|| {
            applications::get_sortable_options(
                &mut matches,
                config.run_cache_weeks,
                config.show_execs,
                config.show_actions_submenu,
                data_dir,
            );
        });
    }
    if let Some(config) = plugins.actions.as_ref() {
        debug_span!("actions").in_scope(|| actions::get_actions_options(&mut matches, config));
    }

    let mut out = vec![];

    if text.is_empty() {
        trace!("text empty, matches: {}", matches.len());
        for r#match in matches {
            out.push((
                r#match.bonus_score.min(20),
                SortedLaunchOption {
                    name: r#match.names[0].clone(),
                    icon: r#match.icon,
                    details: r#match.details,
                    details_long: r#match.details_long,
                    takes_args: r#match.takes_args,
                    enabled: true,
                    iden: r#match.iden,
                    subactions: r#match.subactions,
                },
            ));
        }
        // sort in reverse
        out.sort_by(|(a, _), (b, _)| b.cmp(&a));
        return out;
    }

    let mut config = nucleo::Config::DEFAULT;
    config.prefer_prefix = true;
    let mut matcher = nucleo::Matcher::new(config);
    let pattern = Pattern::parse(
        text,
        nucleo::pattern::CaseMatching::Smart,
        nucleo::pattern::Normalization::Smart,
    );
    let mut buf = Vec::new();

    // TODO add matching of keywords and execs but reduce their scores
    'outer: for r#match in matches {
        // we need some custom matching logic for the ones who take args
        if r#match.takes_args {
            for name in r#match.names {
                if text
                    .trim()
                    .to_ascii_lowercase()
                    .starts_with(&*name.trim().to_ascii_lowercase())
                    && text.trim().len() > name.trim().len()
                {
                    out.push((
                        500,
                        SortedLaunchOption {
                            name,
                            icon: r#match.icon,
                            details: r#match.details,
                            details_long: r#match.details_long,
                            takes_args: r#match.takes_args,
                            enabled: r#match.enabled,
                            iden: r#match.iden,
                            subactions: r#match.subactions,
                        },
                    ));
                    continue 'outer;
                    // TODO check if we can just fzf match
                } else if name
                    .trim()
                    .to_ascii_lowercase()
                    .starts_with(&*text.trim().to_ascii_lowercase())
                    && !text.is_empty()
                {
                    out.push((
                        // because score from fzf increases with more matching chars
                        40 + (text.len() * 30) as u64,
                        SortedLaunchOption {
                            name,
                            icon: r#match.icon,
                            details: r#match.details,
                            details_long: r#match.details_long,
                            takes_args: r#match.takes_args,
                            enabled: false,
                            iden: r#match.iden,
                            subactions: r#match.subactions,
                        },
                    ));
                    continue 'outer;
                }
            }
            continue 'outer;
        }

        let mut maxscore = 0;
        let mut new_score = 0;
        let mut nname = Box::from("");
        for name in r#match.names {
            let nscore = pattern
                .score(nucleo::Utf32Str::new(name.as_ref(), &mut buf), &mut matcher)
                .unwrap_or_default();
            new_score += nscore as u64;
            if nscore >= maxscore {
                nname = name;
                maxscore = nscore;
            }
        }

        if new_score > 10 {
            out.push((
                new_score + r#match.bonus_score.min(20),
                SortedLaunchOption {
                    name: nname,
                    icon: r#match.icon,
                    details: r#match.details,
                    details_long: r#match.details_long,
                    takes_args: r#match.takes_args,
                    enabled: r#match.enabled,
                    iden: r#match.iden,
                    subactions: r#match.subactions,
                },
            ));
        }
    }

    // TODO must be last because curently cant be matched
    let mut matches2 = Vec::new();
    if plugins.path.is_some() {
        debug_span!("path").in_scope(|| path::get_path_options(&mut matches2, text));
    }
    // TODO move these to dynamic plugins
    if plugins.calc.is_some() {
        #[cfg(feature = "calc")]
        debug_span!("calc").in_scope(|| {
            crate::plugins::calc::get_calc_options(&mut matches2, text);
        });
        #[cfg(not(feature = "calc"))]
        tracing::warn!("calc plugin is not enabled");
    }
    if let Some(first) = matches2.pop() {
        out.push((
            20,
            SortedLaunchOption {
                name: first.names[0].clone(),
                icon: first.icon,
                details: first.details,
                details_long: first.details_long,
                takes_args: first.takes_args,
                enabled: first.enabled,
                iden: first.iden,
                subactions: first.subactions,
            },
        ))
    }

    // sort in reverse
    out.sort_by(|(a, _), (b, _)| b.cmp(&a));
    out
}

pub fn get_static_launch_options(
    plugins: &Plugins,
    default_terminal: Option<&str>,
    text: &str,
) -> Vec<StaticLaunchOption> {
    let mut matches = Vec::new();

    if plugins.shell.is_some() {
        debug_span!("shell").in_scope(|| {
            shell::get_static_options(&mut matches, text);
        });
    }
    if plugins.terminal.is_some() {
        debug_span!("terminal").in_scope(|| {
            terminal::get_static_options(&mut matches, default_terminal, text);
        });
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        debug_span!("search").in_scope(|| {
            search::get_static_options(&mut matches, &websearch.engines, text);
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
            actions::run_action(iden.data.as_deref(), text, iden.data_additional.as_deref())
        }),
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
