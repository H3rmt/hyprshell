use crate::cli;
use anyhow::{Context, bail};
use clap_complete::aot;
use std::fs::{create_dir_all, remove_file, write};
use std::path::PathBuf;
use tracing::info;

pub fn generate(shell: &str, path: Option<PathBuf>, delete: bool) -> anyhow::Result<()> {
    use clap::CommandFactory;
    let cli = &mut cli::App::command();
    let mut buffer = Vec::new();
    match shell {
        "bash" => {
            let mut path = path.unwrap_or_else(|| "/usr/share/bash-completion/completions".into());
            create_dir_all(&path)
                .with_context(|| format!("failed to create directory: {}", path.display()))?;
            path.push("hyprshell.bash");
            if delete {
                remove_file(&path)
                    .with_context(|| format!("failed to remove file: {}", path.display()))?;
                info!(
                    "Removed existing bash completion script at: {}",
                    path.display()
                );
            } else {
                aot::generate(aot::Bash, cli, "hyprshell", &mut buffer);
                write(&path, buffer)
                    .with_context(|| format!("failed to write to file: {}", path.display()))?;
                info!("Generated bash completion script at: {}", path.display());
            }
        }
        "zsh" => {
            let mut path = path.unwrap_or_else(|| "/usr/share/zsh/site-functions".into());
            create_dir_all(&path)
                .with_context(|| format!("failed to create directory: {}", path.display()))?;
            path.push("_hyprshell");
            if delete {
                remove_file(&path)
                    .with_context(|| format!("failed to remove file: {}", path.display()))?;
                info!(
                    "Removed existing zsh completion script at: {}",
                    path.display()
                );
            } else {
                aot::generate(aot::Zsh, cli, "hyprshell", &mut buffer);
                write(&path, buffer)
                    .with_context(|| format!("failed to write to file: {}", path.display()))?;
                info!("Generated zsh completion script at: {}", path.display());
            }
        }
        "fish" => {
            let mut path = path.unwrap_or_else(|| "/usr/share/fish/vendor_completions.d".into());
            create_dir_all(&path)
                .with_context(|| format!("failed to create directory: {}", path.display()))?;
            path.push("hyprshell.fish");
            if delete {
                remove_file(&path)
                    .with_context(|| format!("failed to remove file: {}", path.display()))?;
                info!(
                    "Removed existing fish completion script at: {}",
                    path.display()
                );
            } else {
                aot::generate(aot::Fish, cli, "hyprshell", &mut buffer);
                write(&path, buffer)
                    .with_context(|| format!("failed to write to file: {}", path.display()))?;
                info!("Generated fish completion script at: {}", path.display());
            }
        }
        _ => bail!("unknown shell: {shell}"),
    }
    Ok(())
}
