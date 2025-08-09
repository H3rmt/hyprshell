use crate::generate::tui::DEFAULT_COLORS;
use anyhow::{Context, bail};
use std::fs::create_dir_all;
use std::path::Path;
use tracing::{debug_span, info};

#[derive(Debug)]
pub struct StyleData {
    pub default_color: Box<str>,
}

const CSS_CONFIG: &str = include_str!("default.css");

pub fn write_css(css_path: &Path, data: &StyleData, override_file: bool) -> anyhow::Result<()> {
    let _span = debug_span!("write_css").entered();

    if css_path.exists() && !override_file {
        bail!(
            "CSS file at {css_path:?} already exists, delete it before generating a new one or use -f to override"
        );
    }
    if let Some(parent) = css_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({})", parent.display()))?;
    }

    let repl = CSS_CONFIG.replace(
        "(active-color)",
        DEFAULT_COLORS
            .iter()
            .find(|&&(n, _)| *n == *data.default_color)
            .map_or(DEFAULT_COLORS[0].1, |&(_, color)| color),
    );

    std::fs::write(css_path, repl)
        .with_context(|| format!("Failed to write css file at ({})", css_path.display()))?;

    info!("CSS file generated successfully at {}", css_path.display());
    Ok(())
}
