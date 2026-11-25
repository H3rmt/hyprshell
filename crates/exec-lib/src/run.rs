use anyhow::{Context, bail};
use core_lib::TERMINALS;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, thread};
use tracing::{debug, trace, warn};

// use force_same_sysd_unit for things like loginctl terminate-session self
// TODO rename,etc. remove terminal
pub fn run_terminal_program(
    run: &str,
    path: Option<&Path>,
    default_terminal: Option<&str>,
    force_same_sysd_unit: bool,
    cache_dir: &Path,
) -> anyhow::Result<()> {
    debug!("Running: {run}");
    if let Some(term) = default_terminal {
        let command = format!("{term} -e {run}");
        run_program(&command, path, force_same_sysd_unit, cache_dir)
            .context("Failed to run command")?;
    } else {
        let env_path =
            env::var_os("PATH").unwrap_or_else(|| OsString::from("/usr/bin:/bin:/usr/local/bin"));
        debug!(
            "No default terminal found, searching common terminals in PATH. (Set default_terminal in config to avoid this search)"
        );
        trace!("PATH: {}", env_path.to_string_lossy());
        let paths: Vec<_> = env::split_paths(&env_path).collect();
        let mut found_terminal = false;
        for term in TERMINALS {
            if paths.iter().any(|p| p.join(term).exists()) {
                let command = format!("{term} -e {run}");
                if run_program(&command, path, force_same_sysd_unit, cache_dir).is_ok() {
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
    Ok(())
}

pub fn run_program(
    run: &str,
    path: Option<&Path>,
    force_same_sysd_unit: bool,
    cache_dir: &Path,
) -> anyhow::Result<()> {
    trace!("Original command: {run:?}");
    let executable = env::current_exe().context("Failed to locate hyprshell executable")?;
    let systemd_run = env::var_os("INVOCATION_ID").is_some() && !force_same_sysd_unit;

    let internal_run = build_internal_run_command(
        &executable,
        run,
        path,
        cache_dir,
        command_output_logging_enabled(),
        force_same_sysd_unit,
    );
    let mut cmd = if systemd_run {
        build_systemd_run_command(internal_run)
    } else {
        internal_run
    };

    debug!("Running command: {cmd:?}");
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    thread::spawn(move || match child.wait() {
        Ok(status) if status.success() => trace!("Command [{cmd:?}] finished with status {status}"),
        Ok(status) => warn!("Command helper [{cmd:?}] exited with status {status}"),
        Err(error) => debug!("Failed waiting for command [{cmd:?}]: {error}"),
    });
    Ok(())
}

fn build_internal_run_command(
    executable: &Path,
    command: &str,
    working_directory: Option<&Path>,
    cache_dir: &Path,
    store_logs: bool,
    same_process_group: bool,
) -> Command {
    let mut command = command.to_string();
    for replacement in ["%f", "%F", "%u", "%U"] {
        command = command.replace(replacement, "");
    }

    let mut cmd = Command::new(executable);
    cmd.args(["internal", "run", "-v", "--log-dir"])
        .arg(cache_dir.join("logs"));
    if store_logs {
        cmd.arg("--store-logs");
    }
    if same_process_group {
        cmd.arg("--same-process-group");
    }
    cmd.arg("--shell");
    if let Some(path) = working_directory {
        cmd.arg("--working-directory").arg(path);
    }
    cmd.arg("--").arg(command);
    cmd
}

fn build_systemd_run_command(internal_run: Command) -> Command {
    let mut cmd = Command::new("systemd-run");
    cmd.args(["--user", "--scope", "--collect", "--no-ask-password", "--"]);
    cmd.arg(internal_run.get_program())
        .args(internal_run.get_args());
    cmd
}

fn command_output_logging_enabled() -> bool {
    env::var("HYPRSHELL_LOG_COMMAND_OUTPUT")
        .ok()
        .is_some_and(|value| is_truthy(&value))
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::{build_internal_run_command, build_systemd_run_command};
    use std::path::Path;
    use std::process::Command;

    fn command_parts(command: &Command) -> (String, Vec<String>) {
        (
            command.get_program().to_string_lossy().into_owned(),
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        )
    }

    #[test]
    fn builds_direct_command_outside_systemd() {
        let command = build_internal_run_command(
            Path::new("/usr/bin/hyprshell"),
            "my-app --option",
            Some(Path::new("/home/user")),
            Path::new("/home/user/.cache/hyprshell"),
            false,
            false,
        );

        assert_eq!(
            command_parts(&command),
            (
                "/usr/bin/hyprshell".to_string(),
                vec![
                    "internal",
                    "run",
                    "-v",
                    "--log-dir",
                    "/home/user/.cache/hyprshell/logs",
                    "--shell",
                    "--working-directory",
                    "/home/user",
                    "--",
                    "my-app --option",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            )
        );
    }

    #[test]
    fn builds_systemd_scope_around_internal_run_command() {
        let internal_run = build_internal_run_command(
            Path::new("/usr/bin/hyprshell"),
            "my-app --option",
            Some(Path::new("/home/user")),
            Path::new("/home/user/.cache/hyprshell"),
            true,
            false,
        );
        let command = build_systemd_run_command(internal_run);

        assert_eq!(
            command_parts(&command),
            (
                "systemd-run".to_string(),
                vec![
                    "--user",
                    "--scope",
                    "--collect",
                    "--no-ask-password",
                    "--",
                    "/usr/bin/hyprshell",
                    "internal",
                    "run",
                    "-v",
                    "--log-dir",
                    "/home/user/.cache/hyprshell/logs",
                    "--store-logs",
                    "--shell",
                    "--working-directory",
                    "/home/user",
                    "--",
                    "my-app --option",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            )
        );
    }

    #[test]
    fn force_same_sysd_unit_is_passed_to_internal_runner() {
        let command = build_internal_run_command(
            Path::new("/usr/bin/hyprshell"),
            "my-app",
            None,
            Path::new("/home/user/.cache/hyprshell"),
            false,
            true,
        );

        assert_eq!(
            command_parts(&command),
            (
                "/usr/bin/hyprshell".to_string(),
                vec![
                    "internal",
                    "run",
                    "-v",
                    "--log-dir",
                    "/home/user/.cache/hyprshell/logs",
                    "--same-process-group",
                    "--shell",
                    "--",
                    "my-app",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            )
        );
    }

    #[test]
    fn removes_desktop_exec_placeholders() {
        let command = build_internal_run_command(
            Path::new("/usr/bin/hyprshell"),
            "open %f %F %u %U",
            None,
            Path::new("/cache"),
            false,
            false,
        );

        assert_eq!(
            command_parts(&command).1,
            vec![
                "internal",
                "run",
                "-v",
                "--log-dir",
                "/cache/logs",
                "--shell",
                "--",
                "open    ",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn parses_command_output_logging_environment_values() {
        for value in ["true", "TRUE", "1", "yes", "on"] {
            assert!(
                super::is_truthy(value),
                "expected {value:?} to enable logging"
            );
        }

        for value in ["", "false", "0", "no", "off", "enabled"] {
            assert!(
                !super::is_truthy(value),
                "expected {value:?} to disable logging"
            );
        }
    }
}
