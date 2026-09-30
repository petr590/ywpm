use serde::{Deserialize, Serialize};

use crate::state::{DisplayMode, TimePeriod};

/// WallpaperNode - узел, который может представлять как файл, так и папку.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WallpaperNode {
    path: String,

    #[serde(
        default = "DisplayMode::new",
        skip_serializing_if = "DisplayMode::is_default",
        with = "crate::state::display_mode_format"
    )]
    pub mode: DisplayMode,

    #[serde(
        default = "default_recursive_level",
        skip_serializing_if = "is_default_recursive_level"
    )]
    pub recursive_level: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<TimePeriod>,
}

const fn default_recursive_level() -> u16 {
    1
}

const fn is_default_recursive_level(recursive_level: &u16) -> bool {
    *recursive_level == default_recursive_level()
}

impl WallpaperNode {
    pub fn new(path: impl Into<String>, mode: DisplayMode, recursive_level: u16, period: Option<TimePeriod>) -> Self {
        Self {
            path: path.into(),
            mode,
            recursive_level,
            period,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}