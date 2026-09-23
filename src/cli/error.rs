use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ArgParseError {
    message: String,
}

impl std::error::Error for ArgParseError {}

impl fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ArgParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

#[macro_export]
macro_rules! arg_parse_error_localized {
    ($en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::cli::ArgParseError::new(crate::format_localized!($en_fmt, $ru_fmt $(, $arg)*))
    };
}
