use std::collections::HashSet;
use std::rc::Rc;

use crate::server::file_or_dir::FileOrDir;
use crate::server::dtos::WallpaperGroupDto;

#[derive(Debug, Clone)]
pub struct WallpaperGroup {
    wallpapers: HashSet<Rc<FileOrDir>>,
}

impl WallpaperGroup {

    pub fn new() -> Self {
        Self { wallpapers: HashSet::new() }
    }

    pub fn add(&mut self, file_or_dir: Rc<FileOrDir>) {
        self.wallpapers.insert(file_or_dir);
    }

    pub fn as_dto(&self) -> WallpaperGroupDto {
        WallpaperGroupDto {
            wallpapers: self.wallpapers.iter()
                .map(|file_or_dir| file_or_dir.path().to_string())
                .collect()
        }
    }
}