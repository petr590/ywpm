mod display_mode;
mod dtos;
mod state;
mod time_period;
mod wallpaper;
mod wallpaper_group;
mod wallpaper_node;

pub use state::State;

pub(crate) use display_mode::{AlignX, AlignY, DisplayMode, FitMode};
pub(crate) use time_period::TimePeriod;
pub(crate) use wallpaper::Wallpaper;
pub(crate) use wallpaper_group::WallpaperGroup;
pub(crate) use wallpaper_node::{SharedWallpaperNode, WallpaperNode};

#[cfg(not(test))]
mod display_mode_format;

#[cfg(test)]
pub(crate) mod display_mode_format;

#[cfg(test)]
pub(crate) use display_mode::DisplayModeParseError;