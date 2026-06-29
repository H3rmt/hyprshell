/// An unexpected response from interacting with hyprpaper.
#[derive(Debug, derive_more::Display)]
pub enum Error {
    /// The keyword was not executed correctly, for example by misformed input
    /// or a path that does not exist.
    NotOk(String),
    /// When we failed to parse the active wallpapers response from hyprpaper.
    FailedToParseActiveWallpapers(String),
    /// There are no active wallpapers when asking list the active ones.
    NoWallpapersActive,
    /// There are no loaded wallpapers when asking list the loaded ones.
    NoWallpapersLoaded,
}

impl std::error::Error for Error {}
