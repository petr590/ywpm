use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{SharedWallpaperNode, TimePeriod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperGroupDto {
    #[serde(rename = "wallpapers")]
    pub nodes: Vec<PathBuf>,

    pub period: Option<TimePeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDto {
    pub current_wallpaper_path: Option<PathBuf>,

    #[serde(rename = "wallpapers")]
    pub nodes: Vec<SharedWallpaperNode>,

    pub groups: HashMap<String, WallpaperGroupDto>,
}
