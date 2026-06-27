use std::fmt;

#[derive(Debug)]
pub enum PackageError {
    EmptyPackage,
    CorruptedData,
    CorruptedUtf8String,
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageError::EmptyPackage        => write!(f, "Recieved package with 0 size"),
            PackageError::CorruptedData       => write!(f, "Package data is corrupted"),
            PackageError::CorruptedUtf8String => write!(f, "Unable to parse UTF-8 string"),
        }
    }
}

impl std::error::Error for PackageError {}