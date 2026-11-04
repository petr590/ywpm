use serde::Deserialize;

use std::error::Error;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::config::ConfigParseError;
use crate::{format_localized, localized};

#[derive(Debug, Clone)]
pub struct Config {
    state_path:     String,
    images_command: Option<String>,
    video_command:  Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigDto {
    state_path:     String,
    images_command: Option<String>,
    video_command:  Option<String>,
    command:        Option<String>,
}

impl TryFrom<ConfigDto> for Config {
    type Error = ConfigParseError;

    fn try_from(dto: ConfigDto) -> Result<Self, Self::Error> {

		let is_img_cmd_set = dto.images_command.is_some();
		let is_vid_cmd_set = dto.video_command.is_some();

		if dto.command.is_some() && (is_img_cmd_set || is_vid_cmd_set) {
			
			let params = match (is_img_cmd_set, is_vid_cmd_set) {
				(true, false) => "'images-command'",
				(false, true) => "'video-command'",

				_  => localized!(
					"'images-command' and 'video-command'",
					"'images-command' и 'video-command'"
				)
			};

			return Err(ConfigParseError::new(format_localized!(
				"Cannot set {params} when 'command' parameter is set",
				"Невозможно задать {params}, если задан параметр 'command'"
			)));
		}

		if dto.command.is_some() {
			Ok(Config {
				state_path:     dto.state_path,
				images_command: dto.command.clone(),
				video_command:  dto.command,
			})
		} else {
			Ok(Config {
				state_path:     dto.state_path,
				images_command: dto.images_command,
				video_command:  dto.video_command,
			})
		}
    }
}


impl Config {
	pub fn new(state_path: impl Into<String>, images_command: Option<String>, video_command: Option<String>) -> Self {
		Self {
			state_path: state_path.into(),
			images_command,
			video_command
		}
	}

	pub fn read_first_existing(paths: &[impl AsRef<Path>]) -> Result<Self, Box<dyn Error>> {
		if paths.is_empty() {
			panic!("Config::read_first_existing: paths must not be empty!");
		}

		let mut result = Self::read(&paths[0]);
		
		for i in 1..paths.len() {
			if result.is_ok() {
				break;
			}

			result = Self::read(&paths[i]);
		}

		return result;
	}

	pub fn read(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
		let path = path.as_ref();
		let content = wrap_not_found_error(fs::read_to_string(path), path)?;
		let dto: ConfigDto = yaml_serde::from_str(&content)?;
		Ok(Self::try_from(dto)?)
	}

	pub fn state_path(&self) -> &str {
		&self.state_path
	}

	pub fn images_command(&self) -> &Option<String> {
		&self.images_command
	}

	pub fn video_command(&self) ->  &Option<String> {
		&self.video_command
	}
}

fn wrap_not_found_error<T>(result: Result<T, io::Error>, path: &Path) -> Result<T, io::Error> {
	match result {
		Err(err) if err.kind() == ErrorKind::NotFound => {
			Err(io::Error::new(
				ErrorKind::NotFound,
				format!("No such file or directory: {}", path.to_str().unwrap_or_default())
			))
		}

		result => result,
	}
}