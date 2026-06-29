use super::Error;
use crate::error::HyprError;

/// A listing of an active wallpaper.
#[derive(Debug, PartialEq)]
pub struct WallpaperListing {
    /// The monitor that the wallpaper is active on, or `None` if no specific
    /// monitor.
    pub monitor: Option<String>,
    /// The active wallpaper.
    pub wallpaper_path: String,
}

impl TryFrom<&str> for WallpaperListing {
    type Error = HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (monitor, wallpaper_path) = s.split_once('=').ok_or_else(|| {
            HyprError::Hyprpaper(Error::FailedToParseActiveWallpapers(s.to_owned()))
        })?;
        let monitor = {
            let monitor = monitor.trim();
            let has_monitor = !monitor.is_empty();
            has_monitor.then(|| monitor.to_owned())
        };
        let wallpaper_path = wallpaper_path.trim().to_owned();
        Ok(Self {
            monitor,
            wallpaper_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check_ok(s: &str, expected_monitor: Option<&str>, expected_wallpaper_path: &str) {
        #[allow(clippy::unwrap_used)]
        let wallpaper_listing = WallpaperListing::try_from(s).unwrap();
        assert_eq!(wallpaper_listing.monitor.as_deref(), expected_monitor);
        assert_eq!(wallpaper_listing.wallpaper_path, expected_wallpaper_path);
    }

    #[test]
    fn test_ok_no_monitor() {
        let s = " = /foo/bar.jpg";
        check_ok(s, None, "/foo/bar.jpg");
    }

    #[test]
    fn test_ok_with_monitor() {
        let s = "DP-1 = /foo/bar.jpg";
        check_ok(s, Some("DP-1"), "/foo/bar.jpg");
    }

    #[test]
    fn test_err() {
        let s = "DP-1 /foo/bar.jpg";
        assert!(matches!(
            WallpaperListing::try_from(s),
            Err(HyprError::Hyprpaper(Error::FailedToParseActiveWallpapers(
                err_str
            ))) if s == err_str
        ));
    }
}
