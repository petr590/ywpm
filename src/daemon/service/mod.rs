pub mod config;
pub mod wallpaper;

mod action_performer;
mod group;
mod media;
mod media_fit;
mod node;
mod resolution;

pub use action_performer::perform_action_and_update_config;

pub(crate) use media::{MediaType, get_media_type};

#[cfg(test)]
pub(crate) use resolution::Resolution;