use std::error::Error;
use std::string::FromUtf8Error;
use std::{fmt, io};

#[derive(Debug)]
pub enum ReadWriteError {
    Io(io::Error),
    FromUtf8(FromUtf8Error),
    InvalidResponse(String),
    EmptyPackage,
}

impl Error for ReadWriteError {}

impl fmt::Display for ReadWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}