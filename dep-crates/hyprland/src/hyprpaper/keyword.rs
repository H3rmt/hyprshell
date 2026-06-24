use super::{Error, Preload, Reload, Response, Unload, Wallpaper, WallpaperListing};
use crate::error::HyprError;

/// The hyprpaper keyword, used to interact with hyprpaper.
pub enum Keyword {
    /// Preload a wallpaper into memory.
    Preload(Preload),
    /// Reload hyprpaper with another wallpaper.
    Reload(Reload),
    /// Unload a wallpaper from memory.
    Unload(Unload),
    /// Set an already preloaded wallpaper.
    Wallpaper(Wallpaper),
    /// Request a list of active wallpapers.
    ListActive,
    /// Request a list of loaded wallpapers.
    ListLoaded,
}

pub(super) enum ExpectedResponse {
    Ok,
    Active,
    Loaded,
}

impl ExpectedResponse {
    pub(super) fn is_expected(&self, response: String) -> crate::Result<Response> {
        match self {
            ExpectedResponse::Ok => {
                if response.trim() == "ok" {
                    Ok(Response::Ok)
                } else {
                    Err(HyprError::Hyprpaper(Error::NotOk(response)))
                }
            }
            ExpectedResponse::Active => {
                if response.trim() == "no wallpapers active" {
                    return Err(HyprError::Hyprpaper(Error::NoWallpapersActive));
                }
                let wallpaper_listings = response
                    .lines()
                    .map(WallpaperListing::try_from)
                    .collect::<crate::Result<_>>()?;
                Ok(Response::ActiveWallpapers(wallpaper_listings))
            }
            ExpectedResponse::Loaded => {
                if response.trim() == "no wallpapers loaded" {
                    return Err(HyprError::Hyprpaper(Error::NoWallpapersLoaded));
                }
                let wallpapers = response.lines().map(ToOwned::to_owned).collect();
                Ok(Response::LoadedWallpapers(wallpapers))
            }
        }
    }
}

impl Keyword {
    pub(super) fn expected_response(&self) -> ExpectedResponse {
        match &self {
            Keyword::Preload(_)
            | Keyword::Reload(_)
            | Keyword::Unload(_)
            | Keyword::Wallpaper(_) => ExpectedResponse::Ok,
            Keyword::ListActive => ExpectedResponse::Active,
            Keyword::ListLoaded => ExpectedResponse::Loaded,
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preload(preload) => write!(f, "{preload}"),
            Self::Reload(reload) => write!(f, "{reload}"),
            Self::Unload(unload) => write!(f, "{unload}"),
            Self::Wallpaper(wallpaper) => write!(f, "{wallpaper}"),
            Self::ListActive => write!(f, "listactive"),
            Self::ListLoaded => write!(f, "listloaded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hyprpaper::*;

    #[track_caller]
    fn check(command: Keyword, expected_string: &str) {
        let actual = command.to_string();
        assert_eq!(&actual, expected_string);
    }

    #[test]
    fn test_preload_string() {
        let command = Keyword::Preload(Preload {
            path: "/foo/bar".into(),
        });
        check(command, "preload /foo/bar");
    }

    #[test]
    fn test_wallpaper() {
        let command = Keyword::Wallpaper(Wallpaper {
            monitor: None,
            mode: None,
            path: "/foo/bar".into(),
        });
        check(command, "wallpaper ,/foo/bar");

        let command = Keyword::Wallpaper(Wallpaper {
            monitor: Some(Monitor::Port("DP-1".into())),
            mode: None,
            path: "/foo/bar".into(),
        });
        check(command, "wallpaper DP-1,/foo/bar");

        let command = Keyword::Wallpaper(Wallpaper {
            monitor: Some(Monitor::Description("some monitor desc".into())),
            mode: None,
            path: "/foo/bar".into(),
        });
        check(command, "wallpaper desc:some monitor desc,/foo/bar");

        let command = Keyword::Wallpaper(Wallpaper {
            monitor: Some(Monitor::Description("some monitor desc".into())),
            mode: Some(WallpaperMode::Contain),
            path: "/foo/bar".into(),
        });
        check(command, "wallpaper desc:some monitor desc,contain:/foo/bar");

        let command = Keyword::Wallpaper(Wallpaper {
            monitor: Some(Monitor::Description("some monitor desc".into())),
            mode: Some(WallpaperMode::Tile),
            path: "/foo/bar".into(),
        });
        check(command, "wallpaper desc:some monitor desc,tile:/foo/bar");
    }

    #[test]
    fn test_unload() {
        let command = Keyword::Unload(Unload::Path("/foo/bar".into()));
        check(command, "unload /foo/bar");

        let command = Keyword::Unload(Unload::All);
        check(command, "unload all");
    }
}
