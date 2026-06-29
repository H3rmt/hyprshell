use super::{Monitor, WallpaperMode};

/// Reload hyprpaper with the given wallpaper configuration, effectively swap
/// the wallpaper with this new one.
///
/// This is equivalent to the "preload, set new, unload old" process.
pub struct Reload {
    /// The monitor on which to apply the new wallpaper.
    ///
    /// All monitors, if `None`.
    pub monitor: Option<Monitor>,
    /// The wallpaper mode (how it will fill the screen).
    pub mode: Option<WallpaperMode>,
    /// Path to the wallpaper.
    pub path: String,
}

impl std::fmt::Display for Reload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reload ")?;
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
    fn check(reload: Reload, expected: &str) {
        assert_eq!(reload.to_string(), expected);
    }

    #[test]
    fn test_reload_format_no_monitor_no_mode() {
        let reload = Reload {
            monitor: None,
            mode: None,
            path: "/foo/bar.jpg".into(),
        };
        check(reload, "reload ,/foo/bar.jpg");
    }

    #[test]
    fn test_reload_format_with_monitor_with_mode() {
        let reload = Reload {
            monitor: Some(Monitor::Port("DP-1".into())),
            mode: Some(WallpaperMode::Contain),
            path: "/foo/bar.jpg".into(),
        };
        check(reload, "reload DP-1,contain:/foo/bar.jpg");
    }
}
