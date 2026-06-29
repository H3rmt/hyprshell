use super::{Monitor, WallpaperMode};

/// Set a wallpaper, optionally on a specific monitor.
pub struct Wallpaper {
    /// The monitor on which to apply the wallpaper.
    ///
    /// All monitors, if `None`.
    pub monitor: Option<Monitor>,
    /// The wallpaper mode (how it will fill the screen).
    pub mode: Option<WallpaperMode>,
    /// Path to the wallpaper.
    pub path: String,
}

impl std::fmt::Display for Wallpaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wallpaper ")?;
        if let Some(monitor) = &self.monitor {
            write!(f, "{monitor}")?;
        }
        write!(f, ",")?;
        if let Some(mode) = &self.mode {
            write!(f, "{mode}")?;
        }
        write!(f, "{}", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check(wallpaper: Wallpaper, expected: &str) {
        assert_eq!(wallpaper.to_string(), expected);
    }

    #[test]
    fn test_wallpaper_no_monitor_no_mode() {
        let wallpaper = Wallpaper {
            monitor: None,
            mode: None,
            path: "/foo/bar.jpg".into(),
        };
        check(wallpaper, "wallpaper ,/foo/bar.jpg");
    }

    #[test]
    fn test_wallpaper_with_monitor_with_mode() {
        let wallpaper = Wallpaper {
            monitor: Some(Monitor::Port("DP-1".into())),
            mode: Some(WallpaperMode::Tile),
            path: "/foo/bar.jpg".into(),
        };
        check(wallpaper, "wallpaper DP-1,tile:/foo/bar.jpg");
    }
}
