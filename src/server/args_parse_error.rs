use std::fmt;

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

impl fmt::Display for ArgsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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