use crate::style::ThemeData;
use crate::style::structs::Theme;
use anyhow::{Context, bail};
use core_lib::ini::IniFile;
use std::fs;
use std::path::Path;
use tracing::{debug, instrument, warn};

#[instrument(level = "debug")]
pub fn load_themes(
    path: &Path,
    current_css: &str,
) -> anyhow::Result<(Vec<Theme>, Vec<anyhow::Error>)> {
    let mut themes = Vec::new();
    if !path.exists() {
        bail!("Themes directory does not exist: {}", path.display());
    }

    let mut errors = Vec::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("Failed to read themes directory ({})", path.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                warn!("Failed to read theme directory in: {}", path.display());
                errors.push(err.into());
                continue;
            }
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(err) => {
                warn!(
                    "Failed to get file type for theme directory: {}",
                    entry.path().display()
                );
                errors.push(err.into());
                continue;
            }
        };
        if !file_type.is_dir() {
            warn!("Invalid theme directory: {}", entry.path().display());
            errors.push(anyhow::anyhow!(
                "Invalid theme directory: {}",
                entry.path().display()
            ));
            continue;
        }
        let dir_path = entry.path();
        let style_path = dir_path.join("style.css");
        if style_path.is_file() {
            if let Some(name) = dir_path.file_name().and_then(|n| n.to_str()) {
                let Ok(theme_content) = fs::read_to_string(&style_path) else {
                    warn!("Failed to read theme file: {}", style_path.display());
                    errors.push(anyhow::anyhow!(
                        "Failed to read theme file: {}",
                        style_path.display()
                    ));
                    continue;
                };
                let data_path = dir_path.join("data.ini");
                let Ok(data) = fs::read_to_string(&data_path) else {
                    warn!("Failed to read theme data file: {}", data_path.display());
                    errors.push(anyhow::anyhow!(
                        "Failed to read theme data file: {}",
                        data_path.display()
                    ));
                    continue;
                };

                let data = parse_data(&data, name);
                let image_path = dir_path.join("image.png");
                themes.push(Theme {
                    is_current: theme_content == current_css,
                    name: name.to_string(),
                    path: dir_path.clone(),
                    style: theme_content,
                    data,
                    image_path: if image_path.exists() {
                        Some(image_path)
                    } else {
                        debug!("Image file not found: {}", image_path.display());
                        None
                    },
                });
            }
        } else {
            warn!(
                "Invalid theme directory: {}, style file missing",
                dir_path.display()
            );
            errors.push(anyhow::anyhow!(
                "Invalid theme directory: {}, style file missing",
                dir_path.display()
            ));
        }
    }

    themes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok((themes, errors))
}

fn parse_data(data: &str, name: &str) -> ThemeData {
    let data = IniFile::from_str(data);
    ThemeData {
        name: data
            .get_section("")
            .and_then(|s| s.get_first("name"))
            .unwrap_or(name)
            .to_string(),
        description: data
            .get_section("")
            .and_then(|s| s.get_first("description"))
            .unwrap_or("")
            .replace("\\n", "\n"),
        experimental: data
            .get_section("")
            .and_then(|s| s.get_first("experimental"))
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap_or(false),
    }
}
