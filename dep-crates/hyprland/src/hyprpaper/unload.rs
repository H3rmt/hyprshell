/// Unload a wallpaper from memory.
pub enum Unload {
    /// Unload the wallpaper at this path.
    Path(String),
    /// Unload all wallpapers.
    All,
}

impl std::fmt::Display for Unload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unload ")?;
        match self {
            Self::Path(path) => write!(f, "{path}"),
            Self::All => write!(f, "all"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check(unload: Unload, expected: &str) {
        assert_eq!(unload.to_string(), expected);
    }

    #[test]
    fn test_unload_path() {
        let unload = Unload::Path("/foo/bar.jpg".into());
        check(unload, "unload /foo/bar.jpg");
    }

    #[test]
    fn test_unload_all() {
        let unload = Unload::All;
        check(unload, "unload all");
    }
}
