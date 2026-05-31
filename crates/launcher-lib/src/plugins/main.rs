use crate::plugins::{actions, applications, path, search, shell, terminal};
use config_lib::Plugins;
use core_lib::transfer::{Identifier, PluginName};
use nucleo::pattern::Pattern;
use relm4::adw::gtk::gdk::Key;
use std::path::Path;
use tracing::{debug_span, trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedText {
    pub text: Box<str>,
    pub spans: Vec<TextSpan>,
}

#[derive(Debug, Clone)]
pub struct LaunchItem {
    pub names: Box<[Box<str>]>,
    pub keywords: Box<[Box<str>]>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub bonus_score: u64,
    pub takes_args: bool,
    pub enabled: bool,
    pub iden: Identifier,
    pub children: Box<[LaunchChildItem]>,
}

#[derive(Debug, Clone)]
pub struct LaunchChildItem {
    pub name: Box<str>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub enabled: bool,
    pub iden: Identifier,
}

#[derive(Debug, Clone)]
pub struct MatchedLaunchItem {
    pub item: LaunchItem,
    pub display_name: HighlightedText,
    pub matched_alias: Option<Box<str>>,
    pub arg_text: Option<Box<str>>,
    pub score: u64,
}

#[derive(Debug, Clone)]
pub struct StaticLaunchItem {
    pub text: Box<str>,
    pub details: Box<str>,
    pub icon: Option<Box<Path>>,
    pub key: char,
    pub iden: Identifier,
    pub enabled: bool,
}

fn spans_from_char_indices(text: &str, indices: &[u32]) -> Vec<TextSpan> {
    let mut positions = indices
        .iter()
        .copied()
        .map(|idx| idx as usize)
        .collect::<Vec<_>>();
    if positions.is_empty() {
        return Vec::new();
    }
    positions.sort_unstable();
    positions.dedup();

    let chars = text
        .char_indices()
        .map(|(start, ch)| (start as u32, (start + ch.len_utf8()) as u32))
        .collect::<Vec<_>>();
    let mut spans = Vec::new();
    let mut start = positions[0];
    let mut prev = positions[0];

    for &pos in &positions[1..] {
        if pos == prev + 1 {
            prev = pos;
            continue;
        }
        if let (Some((span_start, _)), Some((_, span_end))) = (chars.get(start), chars.get(prev)) {
            spans.push(TextSpan {
                start: *span_start,
                end: *span_end,
            });
        }
        start = pos;
        prev = pos;
    }

    if let (Some((span_start, _)), Some((_, span_end))) = (chars.get(start), chars.get(prev)) {
        spans.push(TextSpan {
            start: *span_start,
            end: *span_end,
        });
    }

    spans
}

fn highlighted_text(text: Box<str>, indices: &[u32]) -> HighlightedText {
    HighlightedText {
        spans: spans_from_char_indices(&text, indices),
        text,
    }
}

fn consume_alias_prefix(text: &str, alias: &str) -> Option<usize> {
    let alias = alias
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if alias.is_empty() {
        return None;
    }

    let mut alias_idx = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        if ch.is_whitespace() {
            continue;
        }
        if alias_idx >= alias.len() || ch.to_ascii_lowercase() != alias[alias_idx] {
            return None;
        }
        alias_idx += 1;
        if alias_idx == alias.len() {
            return Some(byte_idx + ch.len_utf8());
        }
    }
    None
}

fn extract_action_args(text: &str, alias: &str) -> Option<Box<str>> {
    let text = text.trim_start();
    let consumed = consume_alias_prefix(text, alias)?;
    let args = text[consumed..].trim_start();
    (!args.is_empty()).then(|| args.into())
}

fn score_text(name: &str, query: &str) -> Option<(u64, Vec<u32>)> {
    let mut config = nucleo::Config::DEFAULT;
    config.prefer_prefix = true;
    let mut matcher = nucleo::Matcher::new(config);
    let pattern = Pattern::parse(
        query,
        nucleo::pattern::CaseMatching::Smart,
        nucleo::pattern::Normalization::Smart,
    );
    let mut buf = Vec::new();
    let mut indices = Vec::new();
    let score = pattern.indices(nucleo::Utf32Str::new(name, &mut buf), &mut matcher, &mut indices)?;
    Some((score as u64, indices))
}

fn score_keywords(keywords: &[Box<str>], query: &str) -> Option<(u64, Box<str>, Vec<u32>)> {
    let mut best: Option<(u64, Box<str>, Vec<u32>)> = None;
    for keyword in keywords {
        if let Some((score, indices)) = score_text(keyword.as_ref(), query) {
            let score = score / 2;
            if best.as_ref().is_none_or(|(best_score, _, _)| score > *best_score) {
                best = Some((score, keyword.clone(), indices));
            }
        }
    }
    best
}

fn item_to_match(
    item: LaunchItem,
    display_name: HighlightedText,
    matched_alias: Option<Box<str>>,
    arg_text: Option<Box<str>>,
    score: u64,
) -> MatchedLaunchItem {
    MatchedLaunchItem {
        item,
        display_name,
        matched_alias,
        arg_text,
        score,
    }
}

fn launch_child_to_item(child: &LaunchChildItem) -> LaunchItem {
    LaunchItem {
        names: Box::from([child.name.clone()]),
        keywords: Box::from([]),
        icon: child.icon.clone(),
        details: child.details.clone(),
        details_long: child.details_long.clone(),
        bonus_score: 0,
        takes_args: false,
        enabled: child.enabled,
        iden: child.iden.clone(),
        children: Box::from([]),
    }
}

fn launch_parent_to_item(parent: &LaunchItem) -> LaunchItem {
    let mut item = parent.clone();
    item.children = Box::from([]);
    item
}

fn match_launch_item(item: LaunchItem, text: &str) -> Option<MatchedLaunchItem> {
    let score_bonus = item.bonus_score.min(20);

    if text.is_empty() {
        let name = item.names[0].clone();
        return Some(MatchedLaunchItem {
            item,
            display_name: highlighted_text(name.clone(), &[]),
            matched_alias: Some(name),
            arg_text: None,
            score: 0,
        });
    }

    if item.takes_args {
        for name in &item.names {
            if let Some((score, indices)) = score_text(name, text) {
                return Some(MatchedLaunchItem {
                    item: item.clone(),
                    display_name: highlighted_text(name.clone(), &indices),
                    matched_alias: Some(name.clone()),
                    arg_text: extract_action_args(text, name),
                    score: score + score_bonus,
                });
            }
        }
        return None;
    }

    let mut best_name = item.names[0].clone();
    let mut best_indices = Vec::new();
    let mut best_score = 0u64;

    for name in &item.names {
        if let Some((score, indices)) = score_text(name, text) {
            if score >= best_score {
                best_name = name.clone();
                best_indices = indices;
                best_score = score;
            }
        }
    }

    if let Some((kw_score, kw_name, kw_indices)) = score_keywords(&item.keywords, text) {
        if kw_score >= best_score {
            best_name = kw_name;
            best_indices = kw_indices;
            best_score = kw_score;
        }
    }

    if let Some((details_score, _)) = score_text(&item.details, text) {
        let details_score = details_score / 2;
        if details_score > best_score {
            best_score = details_score;
            best_name = item.names[0].clone();
            best_indices.clear();
        }
    }

    (best_score > 10).then(|| MatchedLaunchItem {
        display_name: highlighted_text(best_name, &best_indices),
        matched_alias: Some(item.names[0].clone()),
        arg_text: None,
        score: best_score + score_bonus,
        item,
    })
}

pub fn get_child_launch_items(parent: &LaunchItem, text: &str) -> Vec<MatchedLaunchItem> {
    let mut items = Vec::with_capacity(parent.children.len() + 1);
    items.push(launch_parent_to_item(parent));
    items.extend(parent.children.iter().map(launch_child_to_item));

    items.into_iter().filter_map(|item| match_launch_item(item, text)).collect()
}

pub fn get_launch_items(plugins: &Plugins, text: &str, data_dir: &Path) -> Vec<MatchedLaunchItem> {
    let mut items = Vec::new();

    if let Some(config) = plugins.applications.as_ref() {
        debug_span!("applications").in_scope(|| {
            applications::get_launch_items(
                &mut items,
                config.run_cache_weeks,
                config.show_execs,
                config.show_actions_submenu,
                data_dir,
            );
        });
    }
    if let Some(config) = plugins.actions.as_ref() {
        debug_span!("actions").in_scope(|| actions::get_launch_items(&mut items, config));
    }

    if text.is_empty() {
        items.sort_by(|a, b| b.bonus_score.cmp(&a.bonus_score));
        return items.into_iter().filter_map(|item| match_launch_item(item, text)).collect();
    }

    items
        .into_iter()
        .filter_map(|item| match_launch_item(item, text))
        .collect()
}

pub fn get_input_driven_launch_items(plugins: &Plugins, text: &str) -> Vec<MatchedLaunchItem> {
    let mut out = Vec::new();

    if plugins.path.is_some() {
        let mut items = Vec::new();
        debug_span!("path").in_scope(|| path::get_launch_items(&mut items, text));
        out.extend(items);
    }

    if plugins.calc.is_some() {
        #[cfg(feature = "calc")]
        debug_span!("calc").in_scope(|| {
            let mut items = Vec::new();
            crate::plugins::calc::get_launch_items(&mut items, text);
            out.extend(items);
        });
        #[cfg(not(feature = "calc"))]
        tracing::warn!("calc plugin is not enabled");
    }

    out.sort_by(|a, b| b.score.cmp(&a.score));
    out
}

pub fn get_static_launch_items(
    plugins: &Plugins,
    default_terminal: Option<&str>,
    text: &str,
) -> Vec<StaticLaunchItem> {
    let mut items = Vec::new();

    if plugins.shell.is_some() {
        debug_span!("shell").in_scope(|| shell::get_static_items(&mut items, text));
    }
    if plugins.terminal.is_some() {
        debug_span!("terminal").in_scope(|| {
            terminal::get_static_options(&mut items, default_terminal, text);
        });
    }
    if let Some(websearch) = plugins.websearch.as_ref() {
        debug_span!("search").in_scope(|| search::get_static_options(&mut items, &websearch.engines, text));
    }

    items
}

pub struct PluginReturn {
    pub show_animation: bool,
}

pub fn launch(iden: &Identifier, text: &str, default_terminal: Option<&str>, data_dir: &Path) -> PluginReturn {
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
        PluginName::Shell => debug_span!("shell").in_scope(|| shell::launch_option(text, default_terminal)),
        PluginName::Terminal => debug_span!("terminal").in_scope(|| terminal::launch_option(text, default_terminal)),
        PluginName::WebSearch => debug_span!("search").in_scope(|| search::launch_option(iden.data.as_deref(), text)),
        PluginName::Path => debug_span!("path").in_scope(|| path::launch_option(text)),
        PluginName::Calc => {
            #[cfg(feature = "calc")]
            debug_span!("calc").in_scope(|| crate::plugins::calc::copy_result(iden.data.as_deref()));
            #[cfg(not(feature = "calc"))]
            tracing::warn!("calc plugin is not enabled");
            PluginReturn { show_animation: false }
        }
        PluginName::Actions => debug_span!("actions").in_scope(|| actions::run_action(iden.data.as_deref(), text, iden.data_additional.as_deref())),
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
