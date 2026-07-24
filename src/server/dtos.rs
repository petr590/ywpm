use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::server::wallpaper_node::SharedWallpaperNode;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperGroupDto {
    pub wallpapers: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDto {
    pub current_wallpaper_path: Option<String>,
    pub wallpapers: Vec<SharedWallpaperNode>,
    pub groups:     HashMap<String, WallpaperGroupDto>,
}