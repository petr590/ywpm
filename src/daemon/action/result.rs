use std::fmt;

use crate::daemon::action::ActionPerformError;
use crate::daemon::warning::Warning;

pub type ActionResult = Result<ActionSuccess, ActionPerformError>;

pub struct ActionSuccess {
    pub message: String,
    pub warning: Warning,
}

impl fmt::Display for ActionSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            self.message, get_ln_if_needed(&self.message),
            self.warning, get_ln_if_needed(self.warning.message()),
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
            warning: Warning::new(),
        }
    }

    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            warning: Warning::new(),
        }
    }

    pub fn with_warning(warning: impl Into<Warning>) -> Self {
        Self {
            message: String::new(),
            warning: warning.into(),
        }
    }
}

impl Into<ActionSuccess> for String {
    fn into(self) -> ActionSuccess {
        ActionSuccess::with_message(self)
    }
}
