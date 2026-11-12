use std::error::Error;
use std::fmt::Display;
use std::path::Path;

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

	pub fn prepend_path(self, path: impl AsRef<Path>) -> Self {
		Self {
			message: format!("{}: {}", path.as_ref().display(), self.message)
		}
	}
}

impl Error for ConfigParseError {}

impl Display for ConfigParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{self:?}")
	}
}