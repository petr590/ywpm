use std::cell::RefMut;

use crate::server::args_parse_error::ArgsParseError;
use crate::server::display_mode::DisplayMode;
use crate::server::wallpaper_node::WallpaperNode;

/// Настройки для WallpaperNode. Каждое значение опционально, так как юзер может задать или не задать определённую настройку.
#[derive(PartialEq)]
pub struct Settings {
    pub mode: Option<DisplayMode>,
    pub recursive_level: Option<u16>,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            mode:            Option::None,
            recursive_level: Option::None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.mode.is_some() || self.recursive_level.is_some()
    }

    pub fn set_mode(&mut self, mode: DisplayMode) -> Result<(), ArgsParseError> {
        if self.mode.is_some() {
            return Err(ArgsParseError::new("More then one '--mode' option specified"));
        }
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