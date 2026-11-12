mod error;
mod result;
mod warning;

pub use error::ActionPerformError;
pub use warning::Warning;
pub(crate) use result::{ActionResult, ActionSuccess};