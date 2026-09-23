pub mod config;
pub mod wallpaper;

pub(crate) mod group;
pub(crate) mod media;
pub(crate) mod node;

mod action_performer;
mod resolution;

pub use action_performer::perform_action_and_update_config;

#[cfg(test)]
pub(crate) use resolution::Resolution;