mod error;
mod result;
mod warning;

pub use error::ActionPerformError;
pub(crate) use result::{ActionResult, ActionSuccess};
pub(crate) use warning::Warning;