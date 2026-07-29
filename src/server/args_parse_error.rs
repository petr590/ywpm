use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct ArgsParseError {
    message: String
}

impl ArgsParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl PartialEq for ArgsParseError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl Display for ArgsParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ArgsParseError {}


#[macro_export]
macro_rules! args_parse_error_format {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        ArgsParseError::new(format!($fmt, $($arg),*))
    };
}