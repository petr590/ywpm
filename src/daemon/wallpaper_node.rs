use std::cell::{RefCell, Ref, RefMut};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::{Serialize, Deserialize};

use crate::daemon::display_mode::DisplayMode;
use crate::daemon::time_period::TimePeriod;

/// WallpaperNode - узел, который может представлять как файл, так и папку.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WallpaperNode {
    pub path: String,

    #[serde(
        default = "DisplayMode::new",
        skip_serializing_if = "DisplayMode::is_default",
        with = "crate::daemon::display_mode_format"
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
        Self { path: path.into(), mode, recursive_level, period }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn with_path(&self, path: impl Into<String>) -> Self {
        Self {
            path:            path.into(),
            mode:            self.mode.clone(),
            recursive_level: self.recursive_level,
            period:          self.period.clone(),
        }
    }
}


#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct SharedWallpaperNode(Rc<RefCell<WallpaperNode>>);

impl PartialEq for SharedWallpaperNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Hash for SharedWallpaperNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl SharedWallpaperNode {
    pub fn new(path: impl Into<String>, mode: DisplayMode, recursive_level: u16, period: Option<TimePeriod>) -> Self {
        Self::from(WallpaperNode::new(path, mode, recursive_level, period))
    }

    pub fn borrow(&self) -> Ref<'_, WallpaperNode> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, WallpaperNode> {
        self.0.borrow_mut()
    }
}

impl From<WallpaperNode> for SharedWallpaperNode {
    fn from(node: WallpaperNode) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }
}