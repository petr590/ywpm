use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::state::{SharedWallpaperNode, TimePeriod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperGroupDto {
    #[serde(rename = "wallpapers")]
    pub nodes: Vec<String>,

    pub period: Option<TimePeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDto {
    pub current_wallpaper_path: Option<String>,

    #[serde(rename = "wallpapers")]
    pub nodes: Vec<SharedWallpaperNode>,

    pub groups: HashMap<String, WallpaperGroupDto>,
}
