use std::rc::Rc;
use serde::{Serialize, Deserialize};

use crate::server::display_mode::DisplayMode;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FileOrDir {
    path: Rc<str>,
    pub mode: DisplayMode,
    pub recursive_level: u16,
}

impl FileOrDir {
    pub fn new(path: impl Into<Rc<str>>, mode: DisplayMode, recursive_level: u16) -> Self {
        Self { path: path.into(), mode, recursive_level }
    }

    pub fn path(&self) -> &Rc<str> {
        &self.path
    }
}