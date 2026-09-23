use std::error::Error;
use std::string::FromUtf8Error;
use std::{fmt, io};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    FromUtf8(FromUtf8Error),
    InvalidResponse(String),
    EmptyPackage,
}

impl Error for ReadError {}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ReadError {
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse(message.into())
    }
}
