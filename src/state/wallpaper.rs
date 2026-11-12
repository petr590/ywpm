use std::path::{Path, PathBuf};

use crate::state::{DisplayMode, WallpaperNode};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Wallpaper {
    /// Путь к обоям. Должен указывать на файл
    pub path: PathBuf,
    pub mode: DisplayMode,
}

impl From<&WallpaperNode> for Wallpaper {
    fn from(node: &WallpaperNode) -> Self {
        Self {
            path: node.path().to_path_buf(),
            mode: node.mode.clone(),
        }
    }
}

impl Wallpaper {
    pub fn new(path: impl Into<PathBuf>, mode: DisplayMode) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn mode(&self) -> &DisplayMode {
        &self.mode
    }
}
