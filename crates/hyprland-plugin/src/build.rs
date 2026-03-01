use anyhow::{Context, bail};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug_span, instrument, trace};

#[instrument(level = "debug", skip_all)]
pub fn build_plugin(dir: &Path, out: &Path) -> anyhow::Result<()> {
    trace!("PATH: {:?}", env::var_os("PATH"));
    trace!("CPATH: {:?}", env::var_os("CPATH"));
    let mut bashcmd = Command::new("bash");
    bashcmd.current_dir(dir).arg("-c").arg(format!(
        // TODO -g -O2
        "gcc -shared -fPIC --no-gnu-unique -std=c++2b -I/usr/include/pixman-1 -o {} -g *.cpp",
        out.display()
    ));

    trace!("Running build command: {bashcmd:?}");
    let out = bashcmd
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
            bail!("Error from [{bashcmd:?}]: {err:?}");
        }
    }
}
