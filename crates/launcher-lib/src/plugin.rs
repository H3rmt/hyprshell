use core_lib::transfer::Identifier;
use nucleo::pattern::Pattern;
use std::path::Path;

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
    pub name: Box<str>,
    pub keywords: Box<[Box<str>]>,
    pub icon: Option<Box<Path>>,
    pub details: Box<str>,
    pub details_long: Option<Box<str>>,
    pub bonus_score: u64,
    pub takes_args: bool,
    pub iden: Identifier,
    pub children: Box<[Self]>,
}

#[derive(Debug, Clone)]
pub enum HighlightElement {
    Name(HighlightedText),
    Keyword(HighlightedText),
    Details(HighlightedText),
    DetailsLong(HighlightedText),
    None,
}

#[derive(Debug, Clone)]
pub struct MatchedLaunchItem {
    pub item: LaunchItem,
    pub highlight: HighlightElement,
    pub score: u64,
    pub enabled: bool,
    pub args: Option<Box<str>>,
}

#[derive(Debug, Clone)]
pub struct PluginItem {
    pub text: Box<str>,
    pub details: Box<str>,
    pub icon: Option<Box<Path>>,
    pub key: char,
    pub iden: Identifier,
}

pub struct PluginReturn {
    pub show_animation: bool,
}

const MIN_SCORE_PER_CHAR: usize = 10;

pub fn highlighted_text(text: Box<str>, indices: &[u32]) -> HighlightedText {
    HighlightedText {
        spans: spans_from_char_indices(&text, indices),
        text,
    }
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

fn extract_action_args(text: &str, alias: &str) -> Option<(usize, Option<Box<str>>)> {
    let alias = alias.trim();
    if alias.is_empty() {
        return None;
    }

    let mut alias_chars = alias.chars().filter(|c| !c.is_whitespace()).peekable();
    let mut last_byte = 0usize;
    let mut seen_match = false;

    for (byte_idx, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if seen_match {
                last_byte = byte_idx + ch.len_utf8();
            }
            continue;
        }

        let mut matched = false;
        while let Some(expected) = alias_chars.peek().copied() {
            if expected.eq_ignore_ascii_case(&ch) {
                alias_chars.next();
                matched = true;
                seen_match = true;
                last_byte = byte_idx + ch.len_utf8();
                break;
            }
            alias_chars.next();
        }

        if !matched {
            return None;
        }

        if alias_chars.peek().is_none() {
            let args = text[last_byte..].trim_start();
            return Some((last_byte, (!args.is_empty()).then(|| args.into())));
        }
    }

    if alias_chars.peek().is_none() && seen_match {
        let args = text[last_byte..].trim_start();
        return Some((last_byte, (!args.is_empty()).then(|| args.into())));
    }

    None
}

/// Get launcher items from a parent item.
pub fn get_child_launch_items_from_parent(parent: &LaunchItem) -> Vec<LaunchItem> {
    let mut items = Vec::with_capacity(parent.children.len() + 1);
    items.push(launch_parent_to_item(parent));
    items.extend(parent.children.clone());
    items
}

pub fn launch_parent_to_item(parent: &LaunchItem) -> LaunchItem {
    let mut item = parent.clone();
    item.children = Box::from([]);
    item
}

pub fn match_launch_item(item: LaunchItem, text: &str) -> Option<MatchedLaunchItem> {
    if text.is_empty() {
        return Some(MatchedLaunchItem {
            item,
            highlight: HighlightElement::None,
            score: 0,
            enabled: true,
            args: None,
        });
    }

    let mut args = None;
    let mut enabled = true;
    if item.takes_args {
        let Some((consumed, extracted_args)) = extract_action_args(text, &item.name) else {
            return Some(MatchedLaunchItem {
                item,
                highlight: HighlightElement::None,
                score: 0,
                enabled: false,
                args: None,
            });
        };
        args = extracted_args;
        enabled = args.is_some();
        let query = text[..consumed].trim_end();

        return score_launch_item(item, query, enabled, args);
    }

    let query = text;

    score_launch_item(item, query, enabled, args)
}

fn score_launch_item(
    item: LaunchItem,
    query: &str,
    enabled: bool,
    args: Option<Box<str>>,
) -> Option<MatchedLaunchItem> {

    let mut best_score = 0u64;
    let mut keyword_name = None;
    let mut name_indices = None;
    let mut keyword_indices = None;
    let mut details_indices = None;
    let mut details_long_indices = None;

    if let Some((score, indices)) = score_text(&item.name, query)
        && score >= best_score
    {
        name_indices = Some(indices);
        best_score = score;
    }

    if let Some((kw_score, kw_name, kw_indices)) = score_keywords(&item.keywords, query)
        && kw_score >= best_score
    {
        keyword_name = Some(kw_name);
        keyword_indices = Some(kw_indices);
        best_score = kw_score;
    }
    if let Some((details_score, dt_indices)) = score_text(&item.details, query) {
        #[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let details_score = ((details_score as f64) * 0.8) as u64;
        if details_score > best_score {
            details_indices = Some(dt_indices);
            best_score = details_score;
        }
    }
    if let Some(dl) = item.details_long.as_ref()
        && let Some((details_score, dt_indices)) = score_text(dl, query)
    {
        #[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let details_score = ((details_score as f64) * 0.7) as u64;
        if details_score > best_score {
            details_long_indices = Some(dt_indices);
            best_score = details_score;
        }
    }

    (best_score > (MIN_SCORE_PER_CHAR * query.len()) as u64).then(|| MatchedLaunchItem {
        highlight: match (
            details_long_indices,
            details_indices,
            keyword_indices,
            name_indices,
        ) {
            (Some(indices), _, _, _) => HighlightElement::DetailsLong(highlighted_text(
                item.details_long
                    .as_ref()
                    .expect("must exist to match")
                    .clone(),
                &indices,
            )),
            (_, Some(indices), _, _) => {
                HighlightElement::Details(highlighted_text(item.details.clone(), &indices))
            }
            (_, _, Some(indices), _) => HighlightElement::Keyword(highlighted_text(
                keyword_name.expect("must be set"),
                &indices,
            )),
            (_, _, _, Some(indices)) => {
                HighlightElement::Name(highlighted_text(item.name.clone(), &indices))
            }
            _ => HighlightElement::None,
        },
        score: best_score + item.bonus_score,
        enabled,
        item,
        args,
    })
}

fn score_text(name: &str, query: &str) -> Option<(u64, Vec<u32>)> {
    let mut config = nucleo::Config::DEFAULT;
    config.prefer_prefix = true;
    let mut matcher = nucleo::Matcher::new(config);
    let pattern = Pattern::parse(
        query,
        nucleo::pattern::CaseMatching::Ignore,
        nucleo::pattern::Normalization::Smart,
    );
    let mut buf = Vec::new();
    let mut indices = Vec::new();
    let score = pattern.indices(
        nucleo::Utf32Str::new(&name.to_ascii_lowercase(), &mut buf),
        &mut matcher,
        &mut indices,
    )?;
    Some((u64::from(score), indices))
}

fn score_keywords(keywords: &[Box<str>], query: &str) -> Option<(u64, Box<str>, Vec<u32>)> {
    let mut best: Option<(u64, Box<str>, Vec<u32>)> = None;
    for keyword in keywords {
        if let Some((score, indices)) = score_text(keyword.as_ref(), query) {
            #[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
            let score = ((score as f64) * 0.75) as u64;
            if best
                .as_ref()
                .is_none_or(|(best_score, _, _)| score > *best_score)
            {
                best = Some((score, keyword.clone(), indices));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::{LaunchItem, extract_action_args, match_launch_item};
    use core_lib::transfer::{Identifier, PluginName};

    #[test]
    fn extract_action_args_handles_fuzzy_prefixes() {
        let cases = [
            ("kill rustrover", "kill", Some("rustrover")),
            ("K ll rustrover", "kill", Some("rustrover")),
            ("kill    rustrover", "kill", Some("rustrover")),
            ("k i l l rustrover", "kill", Some("rustrover")),
            ("kill program rustrover", "kill program", Some("rustrover")),
            ("kill program   rustrover", "kill program", Some("rustrover")),
            ("launch firefox nightly", "launch firefox", Some("nightly")),
            ("open app my-editor", "open app", Some("my-editor")),
        ];

        for (text, alias, expected) in cases {
            assert_eq!(
                extract_action_args(text, alias).map(|(_, a)| a).flatten().as_deref(),
                expected,
                "text={text:?} alias={alias:?}"
            );
        }
    }

    #[test]
    fn match_launch_item_extracts_args() {
        let cases = [
            ("kill", "K ll rustrover", Some("rustrover"), true),
            (
                "kill program",
                "kill program rustrover",
                Some("rustrover"),
                true,
            ),
            (
                "kill program",
                "K ll   program rustrover",
                Some("rustrover"),
                true,
            ),
            (
                "launch firefox",
                "launch firefox nightly",
                Some("nightly"),
                true,
            ),
            ("open app", "open app my-editor", Some("my-editor"), true),
            ("run", "run", None, false),
        ];

        for (name, text, expected_args, expected_enabled) in cases {
            let item = LaunchItem {
                name: name.into(),
                keywords: Box::from([]),
                icon: None,
                details: format!("{name} {{}}").into_boxed_str(),
                details_long: None,
                bonus_score: 0,
                takes_args: true,
                iden: Identifier::data(PluginName::Actions, format!("{name} {{}}").into_boxed_str()),
                children: Box::from([]),
            };

            let matched = match_launch_item(item, text).expect("must match");
            assert_eq!(matched.args.as_deref(), expected_args, "text={text:?}");
            assert_eq!(matched.enabled, expected_enabled, "text={text:?}");
        }
    }
}
