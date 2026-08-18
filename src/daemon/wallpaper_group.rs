use std::collections::{HashMap, HashSet};

use crate::daemon::wallpaper_node::SharedWallpaperNode;
use crate::daemon::dtos::WallpaperGroupDto;

#[derive(Debug, Clone)]
pub struct WallpaperGroup {
    wallpapers: HashSet<SharedWallpaperNode>,
}

impl WallpaperGroup {

    pub fn new() -> Self {
        Self { wallpapers: HashSet::new() }
    }

    pub fn from_dto(dto: &WallpaperGroupDto, wallpapers: &HashMap<String, SharedWallpaperNode>) -> Self {
        Self {
            wallpapers: dto.wallpapers.iter()
                .map(|path| wallpapers.get(path).cloned())
                .flatten().collect()
        }
    }

    pub fn add(&mut self, node: SharedWallpaperNode) {
        self.wallpapers.insert(node);
    }

    pub fn add_all(&mut self, iter: impl IntoIterator<Item = SharedWallpaperNode>) {
        for node in iter {
            self.add(node);
        }
    }

    pub fn as_dto(&self) -> WallpaperGroupDto {
        WallpaperGroupDto {
            wallpapers: self.wallpapers.iter()
                .map(|node| node.borrow().path().to_string())
                .collect()
        }
    }

    pub fn remove(&mut self, path: &str) {
        self.wallpapers.retain(|node| node.borrow().path() == path);
    }

    pub fn clear(&mut self) {
        self.wallpapers.clear();
    }
}