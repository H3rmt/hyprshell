use anyhow::Context;
use std::env;
use std::fmt::Write;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tracing::{debug_span, info, trace};

const UNIT: &str = include_str!("default.service");
pub fn write_systemd_unit(
    config_path: Option<&PathBuf>,
    css_path: Option<&PathBuf>,
    data_dir: Option<&PathBuf>,
    cache_dir: Option<&PathBuf>,
    data_home_dir: &Path,
) -> anyhow::Result<()> {
    let _span = debug_span!("write_systemd_unit").entered();
    let path = {
        let mut data_home_dir = data_home_dir.to_path_buf();
        data_home_dir.push("systemd/user/hyprshell.service");
        data_home_dir
    };

    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({})", parent.display()))?;
    }

    let mut params = String::new();
    if let Some(config_path) = config_path {
        let _ = params.write_str(&format!("--config-file {} ", config_path.display()));
    }
    if let Some(css_path) = css_path {
        let _ = params.write_str(&format!("--css-file {} ", css_path.display()));
    }
    if let Some(data_dir) = data_dir {
        let _ = params.write_str(&format!("--data-dir {} ", data_dir.display()));
    }
    if let Some(cache_dir) = cache_dir {
        let _ = params.write_str(&format!("--cache-dir {} ", cache_dir.display()));
    }

    let extra = if params.is_empty() {
        String::new()
    } else {
        format!("WorkingDirectory={}", env::current_dir()?.to_string_lossy())
    };

    #[allow(clippy::literal_string_with_formatting_args)]
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
    std::fs::write(&path, unit_text).with_context(|| {
        format!(
            "Failed to write Systemd unit file at ({:?})",
            path.display()
        )
    })?;

    info!("Systemd unit file generated successfully at {path:?}");
    Ok(())
}
