use crate::cli::InternalRunArgs;
use anyhow::{Context, bail};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tracing::debug;

struct RunLogFiles {
    run_dir: PathBuf,
    stdout: File,
    stderr: File,
    exit_code: File,
    temporary: bool,
}

pub fn run(args: InternalRunArgs) -> anyhow::Result<()> {
    let mut command = build_command(&args)?;
    if !args.same_process_group {
        command.process_group(0);
    }
    if let Some(path) = &args.working_directory {
        command.current_dir(path);
    }

    create_log_dir(&args.log_dir)?;
    let mut logs = create_run_log_files(
        &args.log_dir,
        &args.command.join(" "),
        &format!("{command:?}"),
        !args.store_logs,
    )?;
    debug!("Internal runner starting command: {command:?}");

    let started_at = Instant::now();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(logs.stdout))
        .stderr(Stdio::from(logs.stderr))
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => {
            if logs.temporary {
                fs::remove_dir_all(&logs.run_dir).with_context(|| {
                    format!(
                        "Failed to remove temporary command logs {}",
                        logs.run_dir.display()
                    )
                })?;
            }
            return Err(error).context("Failed to start command");
        }
    };

    let status = child.wait().context("Failed waiting for command")?;
    let elapsed = started_at.elapsed();
    let exit_code = status.code().unwrap_or_else(|| {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map_or(1, |signal| 128 + signal)
    });
    writeln!(logs.exit_code, "{exit_code}").context("Failed to write command exit code")?;

    if logs.temporary {
        if !status.success() && elapsed <= Duration::from_secs(2) {
            drop(logs.exit_code);
            publish_run_log(&logs.run_dir)?;
        } else {
            let run_dir = logs.run_dir.clone();
            drop(logs.exit_code);
            fs::remove_dir_all(&run_dir).with_context(|| {
                format!(
                    "Failed to remove temporary command logs {}",
                    run_dir.display()
                )
            })?;
        }
    }

    debug!("Internal command finished with status {status}");
    if !status.success() {
        bail!("Command exited with status {status}");
    }
    Ok(())
}

fn build_command(args: &InternalRunArgs) -> anyhow::Result<Command> {
    if args.command.is_empty() {
        bail!("No command was supplied to the internal runner");
    }

    if args.shell {
        let mut command = Command::new("/usr/bin/env");
        command.arg("bash").arg("-c");
        command.arg(&args.command[0]);
        command.args(args.command.iter().skip(1));
        Ok(command)
    } else {
        let mut command = Command::new(&args.command[0]);
        command.args(args.command.iter().skip(1));
        Ok(command)
    }
}

fn escaped_command_prefix(command: &str) -> String {
    let prefix: String = command.chars().take(30).collect();
    let mut escaped = String::new();
    for byte in prefix.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            escaped.push(byte as char);
        } else {
            use std::fmt::Write as _;
            write!(escaped, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    escaped
}

fn create_log_dir(log_dir: &Path) -> anyhow::Result<()> {
    let mut dir_builder = fs::DirBuilder::new();
    dir_builder.recursive(true).mode(0o700);
    dir_builder
        .create(log_dir)
        .with_context(|| format!("Failed to create log directory {}", log_dir.display()))
}

fn create_run_log_files(
    log_dir: &Path,
    command: &str,
    command_debug: &str,
    temporary: bool,
) -> anyhow::Result<RunLogFiles> {
    let process_id = std::process::id();
    let command = escaped_command_prefix(command);
    for _ in 0..16 {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let prefix = if temporary { ".pending" } else { "command" };
        let run_dir = log_dir.join(format!("{prefix}-{timestamp}-{process_id}-{command}"));
        let mut run_dir_builder = fs::DirBuilder::new();
        run_dir_builder.mode(0o700);
        match run_dir_builder.create(&run_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Failed to create command log directory {}",
                        run_dir.display()
                    )
                });
            }
        }

        let mut command_file = create_run_file(&run_dir, "command")?;
        command_file
            .write_all(command_debug.as_bytes())
            .with_context(|| format!("Failed to write command file in {}", run_dir.display()))?;
        let stdout = create_run_file(&run_dir, "stdout")?;
        let stderr = create_run_file(&run_dir, "stderr")?;
        let exit_code = create_run_file(&run_dir, "exit_code")?;
        if !temporary {
            debug!("Storing command logs in {}", run_dir.display());
        }
        return Ok(RunLogFiles {
            run_dir,
            stdout,
            stderr,
            exit_code,
            temporary,
        });
    }
    bail!("Failed to allocate a unique command output log filename")
}

fn publish_run_log(run_dir: &Path) -> anyhow::Result<()> {
    let name = run_dir
        .file_name()
        .context("Temporary command log directory has no filename")?
        .to_string_lossy();
    let suffix = name
        .strip_prefix(".pending-")
        .context("Temporary command log directory has an unexpected name")?;
    let published_dir = run_dir.with_file_name(format!("command-{suffix}"));
    fs::rename(run_dir, &published_dir).with_context(|| {
        format!(
            "Failed to publish command log directory {}",
            published_dir.display()
        )
    })?;
    debug!("Storing command logs in {}", published_dir.display());
    Ok(())
}

fn create_run_file(run_dir: &Path, name: &str) -> anyhow::Result<File> {
    let path = run_dir.join(name);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .with_context(|| format!("Failed to create command log file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::build_command;
    use crate::cli::InternalRunArgs;
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
    fn shell_mode_builds_bash_command() {
        let args = InternalRunArgs {
            log_dir: Path::new("/tmp").to_path_buf(),
            store_logs: false,
            same_process_group: false,
            shell: true,
            working_directory: None,
            command: vec![String::from("echo hi")],
        };

        assert_eq!(
            command_parts(&build_command(&args).unwrap()),
            (
                "/usr/bin/env".to_string(),
                vec!["bash", "-c", "echo hi"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            )
        );
    }

    #[test]
    fn direct_mode_preserves_program_arguments() {
        let args = InternalRunArgs {
            log_dir: Path::new("/tmp").to_path_buf(),
            store_logs: false,
            same_process_group: false,
            shell: false,
            working_directory: None,
            command: vec!["/usr/bin/echo".into(), "hello world".into()],
        };

        assert_eq!(
            command_parts(&build_command(&args).unwrap()),
            ("/usr/bin/echo".to_string(), vec!["hello world".to_string()],)
        );
    }

    #[test]
    fn logged_run_records_command_exit_code_and_output_streams() {
        let cache_dir = std::env::temp_dir().join(format!(
            "hyprshell-internal-run-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let args = InternalRunArgs {
            log_dir: cache_dir.join("logs"),
            store_logs: true,
            same_process_group: false,
            shell: true,
            working_directory: None,
            command: vec![String::from("printf stdout; printf stderr >&2")],
        };

        super::run(args).unwrap();

        let logs_dir = cache_dir.join("logs");
        let run_dir = std::fs::read_dir(&logs_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(
            std::fs::read_to_string(run_dir.join("stdout")).unwrap(),
            "stdout"
        );
        assert_eq!(
            std::fs::read_to_string(run_dir.join("stderr")).unwrap(),
            "stderr"
        );
        assert_eq!(
            std::fs::read_to_string(run_dir.join("exit_code")).unwrap(),
            "0\n"
        );
        let command = std::fs::read_to_string(run_dir.join("command")).unwrap();
        assert!(command.contains("bash"));
        assert!(command.contains("printf stdout"));

        std::fs::remove_dir_all(cache_dir).unwrap();
    }

    #[test]
    fn quick_failure_stores_captured_output_without_store_logs() {
        let log_dir = std::env::temp_dir().join(format!(
            "hyprshell-internal-failure-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let args = InternalRunArgs {
            log_dir: log_dir.clone(),
            store_logs: false,
            same_process_group: false,
            shell: true,
            working_directory: None,
            command: vec![String::from("printf stdout; printf stderr >&2; exit 7")],
        };

        assert!(super::run(args).is_err());

        let run_dir = std::fs::read_dir(&log_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(
            std::fs::read_to_string(run_dir.join("stdout")).unwrap(),
            "stdout"
        );
        assert_eq!(
            std::fs::read_to_string(run_dir.join("stderr")).unwrap(),
            "stderr"
        );
        assert_eq!(
            std::fs::read_to_string(run_dir.join("exit_code")).unwrap(),
            "7\n"
        );

        std::fs::remove_dir_all(log_dir).unwrap();
    }
}
