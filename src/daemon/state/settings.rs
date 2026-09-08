use std::cell::RefMut;

use crate::daemon::state::{DisplayMode, WallpaperNode};

/// Настройки для WallpaperNode. Каждое значение опционально, так как юзер может задать или не задать определённую настройку.
#[derive(Debug, PartialEq, Clone)]
pub struct Settings {
    pub mode: Option<DisplayMode>,
    pub recursive_level: Option<u16>,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            mode: None,
            recursive_level: None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.mode.is_some() || self.recursive_level.is_some()
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn update_node(&self, node: &mut RefMut<WallpaperNode>) {
        if let Some(mode) = &self.mode {
            node.mode = mode.clone();
        }

        if let Some(recursive_level) = self.recursive_level {
            node.recursive_level = recursive_level;
        }
    }
}
