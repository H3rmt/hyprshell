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
    pub enabled: bool,
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

pub fn no_highlight_text(text: Box<str>) -> HighlightedText {
    HighlightedText {
        text,
        spans: vec![],
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

/// Get launcher items from a parent item.
pub fn get_child_launch_items_from_parent(parent: &LaunchItem) -> Vec<LaunchItem> {
    let mut items = Vec::with_capacity(parent.children.len() + 1);
    items.push(launch_parent_to_item(parent));
    items.extend(parent.children.clone());
    items
}

fn extract_action_args(text: &str, alias: &str) -> Option<Box<str>> {
    let text = text.trim_start();
    let consumed = consume_alias_prefix(text, alias)?;
    let args = text[consumed..].trim_start();
    (!args.is_empty()).then(|| args.into())
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
        });
    }

    if item.takes_args {
        // TODO implement takes args
        return None;
    }

    let mut best_score = 0u64;
    let mut keyword_name = None;
    let mut name_indices = None;
    let mut keyword_indices = None;
    let mut details_indices = None;
    let mut details_long_indices = None;

    if let Some((score, indices)) = score_text(&item.name, text)
        && score >= best_score
    {
        name_indices = Some(indices);
        best_score = score;
    }

    if let Some((kw_score, kw_name, kw_indices)) = score_keywords(&item.keywords, text)
        && kw_score >= best_score
    {
        keyword_name = Some(kw_name);
        keyword_indices = Some(kw_indices);
        best_score = kw_score;
    }
    if let Some((details_score, dt_indices)) = score_text(&item.details, text) {
        let details_score = details_score / 2;
        if details_score > best_score {
            details_indices = Some(dt_indices);
            best_score = details_score;
        }
    }
    if let Some(dl) = item.details_long.as_ref()
        && let Some((details_score, dt_indices)) = score_text(dl, text)
    {
        let details_score = details_score / 2;
        if details_score > best_score {
            details_long_indices = Some(dt_indices);
            best_score = details_score;
        }
    }

    (best_score > (MIN_SCORE_PER_CHAR * text.len()) as u64).then(|| MatchedLaunchItem {
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
        item,
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
            let score = score / 2;
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
