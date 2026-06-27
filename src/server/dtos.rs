use std::collections::HashMap;
use std::rc::Rc;
use serde::{Serialize, Deserialize};

use crate::server::file_or_dir::FileOrDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperGroupDto {
    pub wallpapers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDto {
    pub current_wallpaper_path: String,
    pub wallpapers: Vec<Rc<FileOrDir>>,
    pub groups:     HashMap<String, WallpaperGroupDto>,
}