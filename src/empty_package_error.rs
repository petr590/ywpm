use std::fmt;

#[derive(Debug)]
pub struct EmptyPackageError;

impl std::error::Error for EmptyPackageError {}

impl fmt::Display for EmptyPackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Recieved package with zero size")
    }
}