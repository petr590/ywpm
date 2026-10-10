use std::collections::HashMap;

use imagesize::{ImageError, ImageResult};

use crate::core::Warning;
use crate::daemon::service::resolution::Resolution;
use crate::state::{DisplayMode, Wallpaper};

pub enum MediaType {
    Image,
    Video,
}

pub struct MediaInfo {
    pub resolution: Resolution,
    pub mode:       DisplayMode,
    pub media_type: MediaType,
}

impl MediaInfo {
    fn new(resolution: Resolution, mode: DisplayMode, media_type: MediaType) -> Self {
        Self { resolution, mode, media_type }
    }

    pub fn get_for_all<I>(wallpapers: I) -> (HashMap<String, Self>, Warning)
    where
        I: IntoIterator<Item = Wallpaper, IntoIter: ExactSizeIterator>,
    {
        let iter = wallpapers.into_iter();
        let mut info_map = HashMap::with_capacity(iter.len());
        let mut rest = Vec::new();
        let mut warning = Warning::new();

        for wallpaper in iter {
            let path = wallpaper.path();

            match get_image_resolution(wallpaper.path()) {
                Ok(resolution) => {
                    let prev = info_map.insert(
                        wallpaper.path,
                        Self::new(resolution, wallpaper.mode, MediaType::Image)
                    );

                    assert!(prev.is_none());
                }

                Err(ImageError::CorruptedImage) => warning.append_msg_path("Image corrupted", path),
                Err(_) => rest.push(wallpaper),
            }
        }

        for wallpaper in rest {
            match get_video_resolution(wallpaper.path()) {
                Ok(resolution) => {
                    let prev = info_map.insert(
                        wallpaper.path,
                        Self::new(resolution, wallpaper.mode, MediaType::Video)
                    );

                    assert!(prev.is_none());
                }

                Err(warn) => warning.append(warn.message()),
            }
        }

        (info_map, warning)
    }
}

pub(crate) fn get_media_type(path: &str) -> Result<MediaType, Warning> {
    match imagesize::size(path) {
        Ok(_)                           => return Ok(MediaType::Image),
        Err(ImageError::CorruptedImage) => return Err(Warning::with_msg_path("Image corrupted", path)),
        Err(_)                          => {}
    }

    match ffprobe::ffprobe(path) {
        Ok(_)    => Ok(MediaType::Video),
        Err(err) => Err(Warning::with_msg_error_path("ffprobe error", &err, path)),
    }
}

pub(super) fn get_resolution(path: &str) -> Result<Resolution, Warning> {
    match get_image_resolution(path) {
        Ok(resolution)                  => return Ok(resolution),
        Err(ImageError::CorruptedImage) => return Err(Warning::with_msg_path("Image corrupted", path)),
        Err(_)                          => {}
    }

    get_video_resolution(path)
}


fn get_image_resolution(path: &str) -> ImageResult<Resolution> {
    let size = imagesize::size(path)?;

    Ok(Resolution::new(
        size.width as u64,
        size.height as u64
    ))
}

fn get_video_resolution(path: &str) -> Result<Resolution, Warning> {
    let ffprobe = ffprobe::ffprobe(path)
        .map_err(|err| Warning::with_msg_error_path("ffprobe error", &err, path))?;

    let stream = ffprobe.streams.iter()
        .find(|stream| is_codec_type_video(&stream.codec_type));

    match stream {
        Some(stream) => {
            match (stream.width, stream.height) {
                (Some(width), Some(height)) => {
                    Ok(Resolution::new(
                        width as u64,
                        height as u64
                    ))
                }

                _ => {
                    Err(Warning::with_msg_path("Unable to get video resolution", path))
                }
            }
        }

        None => Err(Warning::with_msg_path("File doesn't contain video stream", path)),
    }
}


fn is_codec_type_video(codec_type: &Option<String>) -> bool {
    codec_type.as_ref().is_some_and(|codec_type| codec_type == "video")
}