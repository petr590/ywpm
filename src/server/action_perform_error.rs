use std::fmt;

#[derive(Debug)]
pub struct ActionPerformError {
    message: String
}

impl ActionPerformError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl fmt::Display for ActionPerformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ActionPerformError {}