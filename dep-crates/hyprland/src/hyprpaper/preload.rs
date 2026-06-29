/// Preload a wallpaper into memory.
pub struct Preload {
    /// Path of the wallpaper to preload.
    pub path: String,
}

impl std::fmt::Display for Preload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "preload {}", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload() {
        let preload = Preload {
            path: "/foo/bar.jpg".into(),
        };
        assert_eq!(preload.to_string(), "preload /foo/bar.jpg");
    }
}
