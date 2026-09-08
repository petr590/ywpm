use std::error::Error;
use std::fmt::{self, Write};

pub struct Warning {
    message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl From<String> for Warning {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl Warning {
    pub const fn new() -> Self {
        Self {
            message: String::new(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn append(&mut self, message: &str) {
        let _ = write!(self.message, "{message}");
    }

    pub fn append_ln(&mut self, message: &str) {
        let _ = writeln!(self.message, "{message}");
    }

    pub fn append_msg_error_path(&mut self, message: &str, err: &dyn Error, path: &str) {
        let _ = writeln!(self.message, "Warning: {message}: {err}: '{path}'");
    }

    pub fn append_msg_path(&mut self, message: &str, path: &str) {
        let _ = writeln!(self.message, "Warning: {message}: '{path}'");
    }

    pub fn append_msg_error(&mut self, message: &str, err: &dyn Error) {
        let _ = writeln!(self.message, "Warning: {message}: {err}");
    }
}
