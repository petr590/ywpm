use itertools::Itertools;
use serde::Deserialize;

use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::config::ConfigParseError;
use crate::daemon::service::MediaType;
use crate::{format_localized, localized, util};

#[derive(Debug, Clone)]
pub struct Config {
    state_path:     PathBuf,
    images_command: Option<Rc<str>>,
    video_command:  Option<Rc<str>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigDto {
    state_path:     PathBuf,
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

		if  dto.command.is_none() &&
			dto.images_command.is_none() &&
			dto.video_command.is_none()
		{
			return Err(ConfigParseError::new(format_localized!(
				"One of the parameters must be set: 'command', 'images-command' or 'video-command'",
				"Должен быть установлен один из параметров: 'command', 'images-command' или 'video-command'"
			)));
		}


		let state_path = match util::replace_tilde_with_home(&dto.state_path) {
			Cow::Owned(new_path) => new_path,
			Cow::Borrowed(_)     => dto.state_path,
		};

		let cmd1 = dto.command.map(Rc::from);
		let cmd2 = cmd1.clone();

		let images_command = cmd1.or_else(|| dto.images_command.map(Rc::from));
		let video_command  = cmd2.or_else(|| dto.video_command.map(Rc::from));

		Ok(Config { state_path, images_command, video_command })
    }
}


impl Config {
	pub fn new(state_path: impl Into<PathBuf>, images_command: Option<Rc<str>>, video_command: Option<Rc<str>>) -> Self {
		Self {
			state_path: state_path.into(),
			images_command,
			video_command
		}
	}

	#[track_caller]
	pub fn read_first_existing<P>(paths: &[P]) -> Result<(Self, &P), Box<dyn Error>>
	where
		P: AsRef<Path>
	{
		if paths.is_empty() {
			panic!("Paths must not be empty");
		}
		
		for path in paths {
			let result = Self::read(path);
			
			match result {
				Ok(config) => {
					return Ok((config, path))
				}

				Err(err) => {
					if let Some(io_err) = err.downcast_ref::<io::Error>() &&
						io_err.kind() == ErrorKind::NotFound
					{
						continue;
					} else {
						return Err(err);
					}
				}
			}
		}

		let message = format!(
			"Config not found in:\n{}",
			paths.iter()
				.map(|path| path.as_ref().display())
				.join("\n")
		);

		Err(Box::new(io::Error::new(ErrorKind::NotFound, message)))
	}

	pub fn read(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
		let path = path.as_ref();
		let content = wrap_not_found_error(fs::read_to_string(path), path)?;

		let dto: ConfigDto = yaml_serde::from_str(&content)?;
		let config = Self::try_from(dto)
			.map_err(|err| err.prepend_path(path))?;
		
		Ok(config)
	}

	pub fn state_path(&self) -> &Path {
		&self.state_path
	}

	pub fn images_command(&self) -> Option<&str> {
		self.images_command.as_deref()
	}

	pub fn video_command(&self) -> Option<&str> {
		self.video_command.as_deref()
	}

	pub fn media_type_matcher(&self) -> impl Fn(MediaType) -> bool {
		match (self.images_command.is_some(), self.video_command.is_some()) {
			(true,  true)  => |_| true,
			(false, false) => |_| false,
			(true,  false) => |typ| typ == MediaType::Image,
			(false, true)  => |typ| typ == MediaType::Video,
		}
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