use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::Context;
use tracing::debug;

pub fn bundle(
    tmp_path: &Path,
    output: &str,
    input_flat: Option<&Vec<PathBuf>>,
    input_with_path: Option<&Vec<PathBuf>>,
) -> anyhow::Result<()> {
    // Copy all files and folders inside input_with_path to the temporary directory, preserving their original path.
    for input in input_with_path.into_iter().flatten() {
        if !input.exists() {
            anyhow::bail!("Input path does not exist: {}", input.display());
        }
        if input.is_file() {
            let dest = tmp_path.join(input);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directories for input_with_path file")?;
            }
            fs::copy(input, dest)
                .context("Failed to copy input_with_path file to temporary directory")?;
        } else if input.is_dir() {
            copy_dir_all(input, tmp_path.join(input))
                .context("Failed to copy input_with_path directory to temporary directory")?;
        }
    }
    // Copy all files and folders inside input_flat to the root of the temporary directory, ignoring their original path.
    for input in input_flat.into_iter().flatten() {
        if !input.exists() {
            anyhow::bail!("Input path does not exist: {}", input.display());
        }
        if input.is_file() {
            fs::copy(
                input,
                tmp_path.join(input.file_name().expect("no filename?")),
            )
            .context("Failed to copy input_flat file to temporary directory")?;
        } else if input.is_dir() {
            copy_dir_all(input, tmp_path)
                .context("Failed to copy input_flat directory to temporary directory")?;
        }
    }

    let args = vec![
        "cvfa",
        output,
        "-C",
        tmp_path
            .to_str()
            .context("Failed to convert temporary path to string")?,
        ".",
    ];
    debug!("running tar with args: {args:?}");
    let cmd = process::Command::new("tar")
        .args(args)
        .output()
        .context("Failed to create tar archive")?;

    if !cmd.status.success() {
        anyhow::bail!(
            "tar command failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
