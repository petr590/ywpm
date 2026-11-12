use std::num::NonZeroU16;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::Warning;
use crate::daemon::service::{self, MediaType};
use crate::nzu16;
use crate::state::{DisplayMode, TimePeriod};

/// WallpaperNode - узел, который может представлять как файл, так и папку.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WallpaperNode {
    path: PathBuf,

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
    pub recursive_level: NonZeroU16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<TimePeriod>,
}

const fn default_recursive_level() -> NonZeroU16 {
    nzu16!(1)
}

const fn is_default_recursive_level(recursive_level: &NonZeroU16) -> bool {
    recursive_level.get() == default_recursive_level().get()
}

impl WallpaperNode {
    pub fn new(path: impl Into<PathBuf>, mode: DisplayMode, recursive_level: NonZeroU16, period: Option<TimePeriod>) -> Self {
        Self {
            path: path.into(),
            mode,
            recursive_level,
            period,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get_media_type(&self) -> Result<MediaType, Warning> {
        service::get_media_type(&self.path)
    }
}