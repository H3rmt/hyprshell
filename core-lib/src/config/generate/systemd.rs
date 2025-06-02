use crate::get_data_dir;
use anyhow::Context;
use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;
use tracing::{Level, info, span, trace};

const UNIT: &str = include_str!("default.service");
pub fn write_systemd_unit(
    config_path: Option<&PathBuf>,
    css_path: Option<&PathBuf>,
    data_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "write_systemd_unit").entered();
    let path = get_path();

    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({parent:?})"))?;
    }

    let mut params = String::new();
    if let Some(config_path) = config_path {
        params.push_str(&format!("--config-file {config_path:?} "));
    }
    if let Some(css_path) = css_path {
        params.push_str(&format!("--css-file {css_path:?} "));
    }
    if let Some(data_dir) = data_dir {
        params.push_str(&format!("--data-dir {data_dir:?} "));
    }

    let extra = if params.is_empty() {
        "".to_string()
    } else {
        format!("WorkingDirectory={}", env::current_dir()?.to_string_lossy())
    };

    let unit_text = UNIT
        .replace(
            "{path}",
            &env::current_exe()?
                .to_string_lossy()
                .replace("(deleted)", ""),
        )
        .replace("{params}", &params)
        .replace("{extra}", &extra);

    trace!("writing {unit_text} to {path:?}");
    std::fs::write(&path, unit_text)
        .with_context(|| format!("Failed to write Systemd unit file at ({path:?})"))?;

    info!("Systemd unit file generated successfully at {path:?}");
    Ok(())
}

fn get_path() -> PathBuf {
    let mut data_dir = get_data_dir();
    data_dir.push("systemd/user/hyprshell.service");
    data_dir
}
