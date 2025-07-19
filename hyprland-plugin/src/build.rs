use crate::PLUGIN_OUTPUT_PATH;
use anyhow::{Context, bail};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{Level, span, trace};

pub fn build(dir: &TempDir) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "build", path =? dir.path()).entered();
    trace!("PATH: {:?}", env::var_os("PATH"));
    trace!("CPATH: {:?}", env::var_os("CPATH"));
    let mut cmd = Command::new("gcc");
    cmd.current_dir(dir.path())
        .args(["-shared", "-fPIC", "--no-gnu-unique", "-std=c++2b"])
        .arg("-I/usr/include/pixman-1") // fix for arch?
        .arg("-O2")
        .arg("-o")
        .arg(PLUGIN_OUTPUT_PATH);

    for file in dir.path().read_dir()?.flatten() {
        if file.file_name().to_string_lossy().ends_with(".cpp") {
            cmd.arg(file.path());
        }
    }

    trace!("Running build command: {:?}", cmd);
    let out = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn build process")?;
    let output = out.wait_with_output();
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                trace!("Build output (code: {:?})", output.status.code());
                for line in String::from_utf8(output.stderr).unwrap_or_default().lines() {
                    trace!("{line}");
                }
                bail!("Build failed with exit code: {:?}", output.status.code());
            }
        }
        Err(err) => {
            bail!("Error from [{cmd:?}]: {err:?}");
        }
    }
}
