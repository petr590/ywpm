use crate::daemon::state::{DisplayMode, WallpaperNode};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Wallpaper {
    pub path: String,
    pub mode: DisplayMode,
}

impl From<&WallpaperNode> for Wallpaper {
    fn from(node: &WallpaperNode) -> Self {
        Self {
            path: node.path.clone(),
            mode: node.mode.clone(),
        }
    }
}

impl Wallpaper {
    pub fn new(path: impl Into<String>, mode: DisplayMode) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn mode(&self) -> &DisplayMode {
        &self.mode
    }
}
