use anyhow::{Context, bail};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tracing::{Level, span, trace};

pub fn build<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let _span = span!(Level::TRACE, "build", path =? path.as_ref()).entered();
    trace!("PATH: {:?}", env::var_os("PATH"));
    trace!("CPATH: {:?}", env::var_os("CPATH"));
    let mut cmd = Command::new("gcc");
    cmd.current_dir(path.as_ref())
        .args(["-shared", "-fPIC", "--no-gnu-unique", "-std=c++2b"])
        .arg("-I/usr/include/pixman-1") // fix for arch?
        .arg("-O2")
        .arg("-o")
        .arg("hyprfocus.so")
        .arg("main.cpp");

    let out = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn build process")?;
    let output = out.wait_with_output();
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(path.as_ref().join("hyprfocus.so"))
            } else {
                trace!("Build output (code: {:?})", output.status.code());
                for line in String::from_utf8(output.stderr).unwrap_or_default().lines() {
                    trace!("{line}");
                }
                sleep(Duration::from_secs(15));
                bail!("Build failed with exit code: {:?}", output.status.code());
            }
        }
        Err(err) => {
            bail!("Error from [{cmd:?}]: {err:?}");
        }
    }
}
