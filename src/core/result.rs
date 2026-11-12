use std::fmt;

use crate::core::ActionPerformError;

pub type ActionResult = Result<ActionSuccess, ActionPerformError>;

pub struct ActionSuccess {
    pub message: String,
}

impl fmt::Display for ActionSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}",
            self.message, get_ln_if_needed(&self.message),
        )
    }
}

fn get_ln_if_needed(string: &str) -> &'static str {
    if !string.is_empty() && !string.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

impl ActionSuccess {
    pub const fn new() -> Self {
        Self {
            message: String::new(),
        }
    }

    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Into<ActionSuccess> for &str {
    fn into(self) -> ActionSuccess {
        ActionSuccess::with_message(self)
    }
}

impl Into<ActionSuccess> for String {
    fn into(self) -> ActionSuccess {
        ActionSuccess::with_message(self)
    }
}