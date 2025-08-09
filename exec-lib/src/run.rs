use anyhow::{Context, bail};
use core_lib::TERMINALS;
use std::ffi::OsString;
use std::os::unix::prelude::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, thread};
use tracing::{debug, trace};

pub fn run_program(
    run: &str,
    path: Option<&Path>,
    terminal: bool,
    default_terminal: Option<&str>,
) -> anyhow::Result<()> {
    debug!("Running: {run}");
    if terminal {
        if let Some(term) = default_terminal {
            let command = format!("{term} -e {run}");
            run_command(&command, path).context("Failed to run command")?;
        } else {
            let env_path = env::var_os("PATH")
                .unwrap_or_else(|| OsString::from("/usr/bin:/bin:/usr/local/bin"));
            debug!(
                "No default terminal found, searching common terminals in PATH. (Set default_terminal in config to avoid this search)"
            );
            trace!("PATH: {}", env_path.to_string_lossy());
            let paths: Vec<_> = env::split_paths(&env_path).collect();
            let mut found_terminal = false;
            for term in TERMINALS {
                if paths.iter().any(|p| p.join(term).exists()) {
                    let command = format!("{term} -e {run}");
                    if run_command(&command, path).is_ok() {
                        trace!("Found and launched terminal: {term}");
                        found_terminal = true;
                        break;
                    }
                }
            }
            if !found_terminal {
                bail!("Failed to find a terminal to run the command");
            }
        }
    } else {
        run_command(run, path).context("Failed to run command")?;
    }
    Ok(())
}

fn get_command(command: &str) -> Command {
    // replace common exec placeholders
    let mut command = command.to_string();
    for replacement in ["%f", "%F", "%u", "%U"] {
        command = command.replace(replacement, "");
    }
    // if run as systemd unit all programs exit when not run outside the units cgroup
    if env::var_os("INVOCATION_ID").is_some() {
        let mut cmd = Command::new("systemd-run");
        cmd.args(["--user", "--scope", "--collect", "sh", "-c", &command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &command]);
        cmd
    }
}

fn run_command(run: &str, path: Option<&Path>) -> anyhow::Result<()> {
    trace!("Original command: {:?}", run);
    let mut cmd = get_command(run);
    cmd.process_group(0);
    if let Some(path) = path {
        cmd.current_dir(path);
    }

    debug!("Running command: {:?}", cmd);
    let out = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    thread::spawn(move || {
        let start = std::time::Instant::now();
        let output = out.wait_with_output();
        trace!("Command [{cmd:?}] finished");
        if let Ok(output) = output {
            if start.elapsed().as_secs() < 2 && !output.stdout.is_empty()
                || !output.stderr.is_empty()
            {
                trace!("Output from [{cmd:?}]: {output:?}");
            }
        }
    });
    Ok(())
}
