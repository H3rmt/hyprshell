use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use std::os::unix::fs::PermissionsExt;
use tracing::{debug, info, trace};

pub fn set_version_in_pkgbuild(
    pkgbuild_path: &Path,
    version: &semver::Version,
    dry_run: bool,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(pkgbuild_path).context("failed to read PKGBUILD file")?;
    let mut lines = content
        .lines()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let version_line_index = lines
        .iter()
        .position(|line| line.starts_with("pkgver="))
        .context("failed to find pkgver line in PKGBUILD file")?;
    lines[version_line_index] = format!("pkgver={version}");
    if dry_run {
        info!(
            "Dry run: would update version in {} to {version}. look at trace logs for details",
            pkgbuild_path.display()
        );
        trace!(
            "would write into {}: {}",
            pkgbuild_path.display(),
            lines.join("\n")
        );
    } else {
        fs::write(pkgbuild_path, lines.join("\n")).context("failed to write PKGBUILD file")?;
    }
    Ok(())
}

pub fn get_pkgbuild_name(pkgbuild_path: &std::path::Path) -> anyhow::Result<String> {
    let content = fs::read_to_string(pkgbuild_path).context("failed to read PKGBUILD file")?;
    let lines = content.lines().collect::<Vec<_>>();
    let pkgbuild_path_line = lines
        .iter()
        .find(|line| line.starts_with("pkgname="))
        .context("failed to find pkgname line in PKGBUILD file")?;
    let pkgname = pkgbuild_path_line
        .split('=')
        .nth(1)
        .context("failed to parse pkgname line in PKGBUILD file")?;
    Ok(pkgname.to_string())
}

pub fn clone_pkgbuild_to_tmp(
    name: &str,
    tmp_dir: &std::path::Path,
    private_key: &str,
) -> anyhow::Result<PathBuf> {
    let repo_url = format!("ssh://aur@aur.archlinux.org/{name}.git");
    let key = tmp_dir.join("key");
    let mut key_file =
        File::create(&key).context("failed to create temporary file for private key")?;
    key_file
        .set_permissions(std::fs::Permissions::from_mode(0o600))
        .context("failed to set permissions on temporary private key file")?;
    key_file
        .write_fmt(format_args!("{private_key}\n"))
        .context("failed to write private key to temporary file")?;
    let path = tmp_dir.join(name);
    let output = std::process::Command::new("git")
        .arg("clone")
        .arg("-v")
        .arg("--depth")
        .arg("1")
        .arg(&repo_url)
        .arg(&path)
        .env(
            "GIT_SSH_COMMAND",
            format!("ssh -i {} -o IdentitiesOnly=yes", key.display()),
        )
        .output()
        .context("failed to run git clone command")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git clone command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(path)
}
pub fn update_srcinfo_in_pkgbuild(pkgbuild_dir: &Path) -> anyhow::Result<()> {
    // check if updpkgsums exists, if not use docker to run archlinux image to run makepkg
    let args = ["makepkg --printsrcinfo > .SRCINFO"];
    debug!("running updpkgsums command with args: {:?}", args);
    let output = std::process::Command::new("sh")
        .arg("-c")
        .args(args)
        .current_dir(pkgbuild_dir)
        .output()
        .context("failed to run updpkgsums command")?;
    if !output.status.success() {
        debug!(
            "updpkgsums command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        // try to run in docker
        let uid = get_user_id().context("failed to get user id")?;
        let args = [
            "run",
            "--rm",
            "-v",
            &format!("{}:/app", pkgbuild_dir.display()),
            "archlinux:latest",
            "sh",
            "-c",
            &format!(
                "pacman -Sy --noconfirm --needed pacman-contrib && useradd -m builder -u {uid} && chown -R builder:builder /app && su builder -c 'cd /app && makepkg --printsrcinfo > .SRCINFO'"
            ),
        ];
        debug!("running docker command with args: {:?}", args);
        let output = std::process::Command::new("docker")
            .args(args)
            .output()
            .context("failed to run updpkgsums command in docker")?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "updpkgsums command failed in docker with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}

pub fn update_sha256sums_in_pkgbuild(pkgbuild_dir: &Path) -> anyhow::Result<()> {
    // check if updpkgsums exists, if not use docker to run archlinux image to run updpkgsums
    debug!("running updpkgsums command");
    let output = std::process::Command::new("updpkgsums")
        .current_dir(pkgbuild_dir)
        .output()
        .context("failed to run updpkgsums command")?;
    if !output.status.success() {
        debug!(
            "updpkgsums command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        // try to run in docker
        let uid = get_user_id().context("failed to get user id")?;
        let args = [
            "run",
            "--rm",
            "-v",
            &format!("{}:/app", pkgbuild_dir.display()),
            "archlinux:latest",
            "sh",
            "-c",
            &format!(
                "pacman -Sy --noconfirm --needed pacman-contrib && useradd -m builder -u {uid} && chown -R builder:builder /app && su builder -c 'cd /app && updpkgsums'"
            ),
        ];
        debug!("running docker command with args: {:?}", args);
        let output = std::process::Command::new("docker")
            .args(args)
            .output()
            .context("failed to run updpkgsums command in docker")?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "updpkgsums command failed in docker with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}

fn get_user_id() -> anyhow::Result<u32> {
    let output = std::process::Command::new("id")
        .arg("-u")
        .output()
        .context("failed to run id command")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "id command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let uid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .context("failed to parse uid from id command output")?;
    Ok(uid)
}

pub fn commit_and_push_pkgbuild(
    pkgbuild_dir: &Path,
    username: Option<&str>,
    email: Option<&str>,
    msg: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    let args = ["add", "-fv", "PKGBUILD", ".SRCINFO"];
    debug!("running git command with args: {:?}", args);
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(pkgbuild_dir)
        .output()
        .context("failed to run git add command")?;
    trace!(
        "output of git add command: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git add command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let args = ["commit", "-m", msg];
    debug!("running git command with args: {:?}", args);
    let mut envs = std::collections::HashMap::new();
    if let Some(username) = username {
        envs.insert("GIT_AUTHOR_NAME", username);
    }
    if let Some(email) = email {
        envs.insert("GIT_AUTHOR_EMAIL", email);
    }
    let output = std::process::Command::new("git")
        .args(args)
        .envs(envs)
        .current_dir(pkgbuild_dir)
        .output()
        .context("failed to run git commit command")?;
    trace!(
        "output of git add command: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git commit command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !dry_run {
        debug!("running git push command");
        let output = std::process::Command::new("git")
            .arg("push")
            .current_dir(pkgbuild_dir)
            .output()
            .context("failed to run git push command")?;
        trace!(
            "output of git add command: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git push command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}
