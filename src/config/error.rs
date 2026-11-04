use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ConfigParseError {
	message: String
}

impl ConfigParseError {
	pub fn new(message: impl Into<String>) -> Self {
		Self { message: message.into() }
	}

	pub fn message(&self) -> &str {
		&self.message
	}
}

impl Error for ConfigParseError {}

impl Display for ConfigParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{self:?}")
	}
}