mod fs_util;
mod macros;
mod read_write_error;
mod reader;
mod writer;
mod util;

pub use fs_util::*;
pub use read_write_error::ReadWriteError;
pub use reader::*;
pub use writer::*;
pub use util::*;

pub type ReadWriteResult<T> = Result<T, ReadWriteError>;