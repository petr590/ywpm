use std::error::Error;
use std::fmt::{self, Write};
use std::path::Path;

pub struct Warning {
    message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl fmt::Write for Warning {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.message.write_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.message.write_char(c)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.message.write_fmt(args)
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

    pub fn with_msg_path(message: &str, path: impl AsRef<Path>) -> Self {
        let mut warning = Self::new();
        warning.append_msg_path(message, path);
        warning
    }

    pub fn with_msg_error_path(message: &str, err: &dyn Error, path: impl AsRef<Path>) -> Self {
        let mut warning = Self::new();
        warning.append_msg_error_path(message, err, path);
        warning
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn append(&mut self, message: &str) {
        let _ = write!(self.message, "{message}");
    }

    pub fn append_msg_error_path(&mut self, message: &str, err: &dyn Error, path: impl AsRef<Path>) {
        let _ = writeln!(self.message, "Warning: {}: {}: '{}'", message, err, path.as_ref().display());
    }

    pub fn append_msg_path(&mut self, message: &str, path: impl AsRef<Path>) {
        let _ = writeln!(self.message, "Warning: {}: '{}'", message, path.as_ref().display());
    }

    pub fn append_msg_error(&mut self, message: &str, err: &dyn Error) {
        let _ = writeln!(self.message, "Warning: {message}: {err}");
    }
}
