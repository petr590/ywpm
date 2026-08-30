use std::collections::{HashMap, HashSet};

use crate::daemon::time_period::TimePeriod;
use crate::daemon::wallpaper_node::SharedWallpaperNode;
use crate::daemon::dtos::WallpaperGroupDto;

#[derive(Debug, Clone)]
pub struct WallpaperGroup {
    nodes: HashSet<SharedWallpaperNode>,
    pub period: Option<TimePeriod>,
}

impl WallpaperGroup {

    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            period: None
        }
    }

    pub fn from_dto(dto: WallpaperGroupDto, wallpapers: &HashMap<String, SharedWallpaperNode>) -> Self {
        Self {
            nodes: dto.nodes.iter()
                .map(|path| wallpapers.get(path).cloned())
                .flatten().collect(),
            
            period: dto.period
        }
    }

    pub fn nodes(&self) -> &HashSet<SharedWallpaperNode> {
        &self.nodes
    }

    pub fn add(&mut self, node: SharedWallpaperNode) {
        self.nodes.insert(node);
    }

    pub fn add_all(&mut self, nodes: impl IntoIterator<Item = SharedWallpaperNode>) {
        for node in nodes {
            self.add(node);
        }
    }

    pub fn as_dto(&self) -> WallpaperGroupDto {
        WallpaperGroupDto {
            nodes: self.nodes.iter()
                .map(|node| node.borrow().path().to_string())
                .collect(),
            
            period: self.period.clone()
        }
    }

    pub fn remove(&mut self, path: &str) {
        self.nodes.retain(|node| node.borrow().path() == path);
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}