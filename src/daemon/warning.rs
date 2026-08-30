use std::error::Error;
use std::fmt;

pub struct Warning {
    pub message: String
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            Ok(())
        } else {
            writeln!(f, "{}", self.message)
        }
    }
}

impl Warning {

    pub fn new() -> Self {
        Self { message: String::new() }
    }

    pub fn append_msg_error_path(&mut self, message: &str, err: &dyn Error, path: &str) {
        self.message.push_str("Warning: ");
        self.message.push_str(message);
        self.message.push_str(": ");
        self.message.push_str(&err.to_string());
        self.message.push_str(": '");
        self.message.push_str(path);
        self.message.push_str("'\n");
    }

    pub fn append_msg_path(&mut self, message: &str, path: &str) {
        self.message.push_str("Warning: ");
        self.message.push_str(message);
        self.message.push_str(": '");
        self.message.push_str(path);
        self.message.push_str("'\n");
    }

    pub fn append_msg_error(&mut self, message: &str, err: &dyn Error) {
        self.message.push_str("Warning: ");
        self.message.push_str(message);
        self.message.push_str(": ");
        self.message.push_str(&err.to_string());
        self.message.push('\n');
    }
}