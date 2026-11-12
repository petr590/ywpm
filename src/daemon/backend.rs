use std::cell::RefCell;
use std::io;
use std::process::{Child, Command};

use crate::config::Config;
use crate::daemon::service::{self, MediaType, Resolution};
use crate::state::Wallpaper;

thread_local! {
    static CHILD: RefCell<Option<Child>> = RefCell::new(None);
}

pub(crate) fn is_running() -> bool {
    CHILD.with_borrow(|child| child.is_some())
}

pub(crate) fn run(config: &Config, wallpaper: &Wallpaper) -> Result<(), io::Error> {
    stop();

    assert!(wallpaper.path().is_file());

    CHILD.with_borrow_mut(|opt| -> Result<(), io::Error> {
		let path = wallpaper.path();
		let mode = wallpaper.mode();

		let media_type = service::get_media_type(path);

		if let Err(ref warning) = media_type {
			eprintln!("Warning: {warning}");
		}

		let media_type_str = match media_type {
			Ok(MediaType::Image) => "image",
			Ok(MediaType::Video) => "video",
			Err(_)               => "unknown",
		};

		let command = match media_type {
			Ok(MediaType::Image) => config.images_command(),
			Ok(MediaType::Video) => config.video_command(),
			Err(_) => config.images_command()
					.or(config.video_command())
		};

		let Some(command) = command else {
			eprintln!("Backend ran with 'none' command for '{media_type_str}' type");
			*opt = None;
			return Ok(());
		};

		let image_res = service::get_resolution(path)
			.unwrap_or_else(|_| Resolution::new(0, 0));
		
		let display_res = service::get_display_resolution(None)
			.unwrap_or_else(|_| Resolution::new(0, 0));

		let child = Command::new("sh")
			.arg("-c")
			.arg(command)
			.env("FILE_PATH",     wallpaper.path())
			.env("MEDIA_TYPE",    media_type_str)
			.env("FIT_MODE",      format!("{:?}", mode.fit_mode).to_ascii_lowercase())
			.env("ALIGNMENT",     format!("{:?}", mode.alignment).to_ascii_lowercase())
			.env("IMAGE_WIDTH",   image_res.width.to_string())
			.env("IMAGE_HEIGHT",  image_res.height.to_string())
			.env("SCREEN_WIDTH",  display_res.width.to_string())
			.env("SCREEN_HEIGHT", display_res.height.to_string())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.spawn()?;
		
		*opt = Some(child);
		println!("Child process started");

        Ok(())
    })?;

    Ok(())
}

pub fn stop() {
    CHILD.with_borrow_mut(|opt| {
        if let Some(child) = opt {
            match child.kill() {
                Ok(()) => {}
                Err(err) => eprintln!("Error while killing child process: {err}"),
            }

            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Child process returned status: {status}");
                    }
                }

                Err(err) => eprintln!("Error while waiting for child process: {err}"),
            }

			*opt = None;
            println!("Child process stopped");
        }
    });
}
