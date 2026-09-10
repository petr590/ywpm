pub mod config;
pub mod wallpaper;

pub(crate) mod group;
pub(crate) mod media;
pub(crate) mod node;

mod resolution;

#[cfg(test)]
pub(crate) use resolution::Resolution;