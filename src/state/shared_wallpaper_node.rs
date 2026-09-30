use std::cell::{Ref, RefCell, RefMut};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::state::{DisplayMode, TimePeriod, WallpaperNode};


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

impl AsRef<SharedWallpaperNode> for SharedWallpaperNode {
    fn as_ref(&self) -> &SharedWallpaperNode {
        self
    }
}

impl From<WallpaperNode> for SharedWallpaperNode {
    fn from(node: WallpaperNode) -> Self {
        Self(Rc::new(RefCell::new(node)))
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
