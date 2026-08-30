use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ActionPerformError {
    message: String
}

impl Error for ActionPerformError {}

impl fmt::Display for ActionPerformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ActionPerformError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn from_boxed(error: Box<dyn Error>) -> Self {
        match error.downcast::<ActionPerformError>() {
            Ok(action_perform_error) => *action_perform_error,
            Err(error) => ActionPerformError::new(error.to_string())
        }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

#[macro_export]
macro_rules! action_perform_error_localized {
    ($en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::daemon::action_perform_error::ActionPerformError::new(crate::format_localized!($en_fmt, $ru_fmt $(, $arg)*))
    };
}