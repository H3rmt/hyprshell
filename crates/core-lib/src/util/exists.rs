use std::env;
use std::env::split_paths;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Common fallback directories.
const COMMON_DIRS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",
    "/bin",
    "/sbin",
    "/usr/sbin",
    "/usr/local/sbin",
    "/snap/bin",
];
// NixOS-specific locations.
const NIX_DIRS: &[&str] = &[
    "/run/current-system/sw/bin",
    "/nix/var/nix/profiles/default/bin",
];

/// Try to find an executable named `name`.
///
/// - If `name` contains a path separator, that path is checked directly.
/// - Otherwise the function searches:
///   - directories from `PATH`
///   - some common system directories (`/usr/bin`, `/usr/local/bin`, `/bin`, ...)
///   - NixOS-specific locations (`/run/current-system/sw/bin`, `$HOME/.nix-profile/bin`, `/nix/var/nix/profiles/default/bin`)
///
/// Returns `Some(PathBuf)` of the first found executable, or `None`.
#[must_use]
pub fn find_command(name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }

    let path = Path::new(name);

    // If name contains a path separator, check that directly.
    if path.components().count() > 1 {
        return if is_executable(path) {
            Some(path.to_path_buf())
        } else {
            None
        };
    }

    // Collect candidate directories from PATH.
    let env_path = env::var_os("PATH").unwrap_or_default();
    let mut candidates: Vec<_> = split_paths(&env_path).collect();

    for d in COMMON_DIRS {
        candidates.push(PathBuf::from(d));
    }

    for d in NIX_DIRS {
        candidates.push(PathBuf::from(d));
    }

    // User Nix profile if HOME is present.
    if let Some(home) = env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".nix-profile").join("bin"));
    }

    // Check each candidate directory for the executable.
    for dir in candidates {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.clone().join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

/// Convenience wrapper that returns true if the command exists somewhere.
#[must_use]
pub fn command_exists(name: &str) -> bool {
    find_command(name).is_some()
}

fn is_executable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        if let Ok(meta) = fs::metadata(path) {
            let mode = meta.permissions().mode();
            // Check any of the execute bits (owner/group/other).
            return mode & 0o111 != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        // On non-unix platforms, fallback to "exists and is file".
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_shell_exists() {
        // common shell should exist on CI / dev machines; skip strict assertion
        let candidates = ["sh", "bash", "zsh"];
        let found = candidates.iter().any(|c| command_exists(c));
        assert!(found, "expected at least one common shell to exist");
    }

    #[test]
    fn nonexistent_command_does_not_exist() {
        assert!(!command_exists(
            "this-command-should-never-exist-12345-amogus"
        ));
    }
}
