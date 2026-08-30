use std::collections::HashMap;

use display_info::DisplayInfo;
use imagesize::ImageError;
use ffprobe::FfProbeError;
use itertools::Itertools;

use crate::action_perform_error_localized;
use crate::daemon::action_perform_error::ActionPerformError;
use crate::daemon::file_info::{FileInfo, Resolution};
use crate::daemon::warning::Warning;

const EPSILON: f64 = 1e-10;


pub fn run(paths: Vec<String>, display_id: Option<u32>) -> Result<String, ActionPerformError> {
    let display_resolution = get_display_resolution(display_id)?;
    let display_ratio = display_resolution.ratio();

    let (files_info, mut warning) = get_files_info(paths);

    for (path, info) in files_info {
        if info.is_video {
            if info.resolution != display_resolution { 
                warning.append_msg_path(&format!(
                        "video resolution ({}) doesn't match with display resolution ({})",
                        info.resolution, display_resolution
                    ), &path);
            }

        } else {
            if (info.resolution.ratio() - display_ratio).abs() > EPSILON {
                warning.append_msg_path(&format!(
                        "image ratio ({}, {}) doesn't match with display ratio ({}, {})",
                        info.resolution, info.resolution.ratio_str(),
                        display_resolution, display_resolution.ratio_str()
                    ), &path);
            }
        }
    }

    Ok(warning.message)
}


fn get_display_resolution(display_id: Option<u32>) -> Result<Resolution, ActionPerformError> {
    let infos = DisplayInfo::all()
        .map_err(|error| ActionPerformError::new(error.to_string()))?;

    if infos.is_empty() {
        return Err(action_perform_error_localized!(
            "No displays are found",
            "Ни один дисплей не найден"
        ));
    }

    if let Some(display_id) = display_id {
        let info = infos.iter().find(|info| info.id == display_id);

        match info {
            Some(info) => Ok(Resolution::new(info.width, info.height)),

            None => Err(action_perform_error_localized!(
                "Display with ID {display_id} not found",
                "Дисплей с ID {display_id} не найден"
            ))
        }
        
    } else if infos.len() == 1 {
        Ok(Resolution::new(infos[0].width, infos[0].height))
        
    } else {
        Err(action_perform_error_localized!(
            "More than one display was detected. Use --display-id to specify the display id. Available displays: {}",
            "Обнаружено более одного дисплея. Используйте --display-id чтобы указать id дисплея. Доступные дисплеи: {}",
            infos.iter()
                .map(|info| format!("#{} ({}x{})", info.id, info.width, info.height))
                .join(", ")
        ))
    }
}


fn get_files_info(paths: Vec<String>) -> (HashMap<String, FileInfo>, Warning) {
    let mut map = HashMap::with_capacity(paths.len());
    let mut rest = Vec::new();
    let mut warning = Warning::new();

    for path in paths {
        match imagesize::size(&path) {
            Ok(size) => {
                let prev = map.insert(path, FileInfo {
                    resolution: Resolution::new(size.width as u64, size.height as u64),
                    is_video: false,
                });

                assert!(prev.is_none());
            },

            Err(ImageError::IoError(err))   => warning.append_msg_error_path("I/O error", &err, &path),
            Err(ImageError::CorruptedImage) => warning.append_msg_path("Image corrupted", &path),
            Err(ImageError::NotSupported)   => rest.push(path),
        }
    }

    for path in rest {
        match ffprobe::ffprobe(&path) {
            Ok(ffprobe) => {
                let stream = ffprobe.streams.iter()
                    .find(|stream| is_codec_type_video(&stream.codec_type));

                match stream {
                    Some(stream) => {

                        let size = match (stream.width, stream.height) {
                            (Some(width), Some(height)) => FileInfo {
                                resolution: Resolution::new(width as u64, height as u64),
                                is_video: true,
                            },

                            _ => {
                                warning.append_msg_path("Unable to get video resolution", &path);
                                continue;
                            }
                        };
                        
                        let prev = map.insert(path, size);
                        assert!(prev.is_none());
                    }

                    None => warning.append_msg_path("File doesn't contain video stream", &path),
                }
            },

            Err(FfProbeError::Io(err)) => warning.append_msg_error_path("I/O error", &err, &path),
            Err(err)                   => warning.append_msg_error_path("ffprobe error", &err, &path),
        };
    }

    (map, warning)
}

fn is_codec_type_video(codec_type: &Option<String>) -> bool {
    codec_type.as_ref().is_some_and(|codec_type| codec_type == "video")
}