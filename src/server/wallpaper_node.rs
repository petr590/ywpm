use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::hash::{Hash, Hasher};

use serde::{Serialize, Deserialize};

use crate::server::display_mode::DisplayMode;

/// WallpaperNode - узел, который может представлять как файл, так и папку.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WallpaperNode {
    path: String,

    #[serde(with = "crate::server::display_mode_format")]
    pub mode: DisplayMode,

    pub recursive_level: u16,
}

impl WallpaperNode {
    pub fn new(path: impl Into<String>, mode: DisplayMode, recursive_level: u16) -> Self {
        Self { path: path.into(), mode, recursive_level }
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWallpaperNode(Rc<RefCell<WallpaperNode>>);

impl PartialEq for SharedWallpaperNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for SharedWallpaperNode {}

impl Hash for SharedWallpaperNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl SharedWallpaperNode {
    pub fn new(path: impl Into<String>, mode: DisplayMode, recursive_level: u16) -> Self {
        Self::from(WallpaperNode::new(path, mode, recursive_level))
    }

    pub fn from(node: WallpaperNode) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn borrow(&self) -> Ref<'_, WallpaperNode> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, WallpaperNode> {
        self.0.borrow_mut()
    }
}