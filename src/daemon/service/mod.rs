pub mod state_io;
pub mod wallpaper;

mod action_performer;
mod group;
mod media;
mod media_fit;
mod node;
mod resolution;

pub use action_performer::perform_action_and_update_config;

pub(crate) use media::{MediaType, get_media_type, get_resolution};
pub(crate) use media_fit::get_display_resolution;
pub(crate) use resolution::Resolution;