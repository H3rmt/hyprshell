/// The desired fill mode of the wallpaper.
// #[derive(Default)]
pub enum WallpaperMode {
    // /// Cover all of the screen, keeping aspect ratio but potentially cutting of
    // /// at some edges.
    // #[default]
    // Cover,
    /// Contain the wallpaper in the screen, potentially changing aspect ratio.
    Contain,
    /// Tile the wallpaper to fill the entire screen.
    Tile,
}

impl std::fmt::Display for WallpaperMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::Cover => Ok(()), // Default behavior, don't need to write anything.
            Self::Contain => write!(f, "contain:"),
            Self::Tile => write!(f, "tile:"),
        }
    }
}
