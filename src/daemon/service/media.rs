use std::collections::HashMap;
use std::fmt::Write;

use display_info::DisplayInfo;
use ffprobe::FfProbeError;
use imagesize::ImageError;
use indexmap::IndexSet;
use itertools::Itertools;

use crate::action_perform_error_localized;
use crate::core::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::service::resolution::Resolution;
use crate::core::Warning;
use crate::state::{DisplayMode, SharedWallpaperNode, State, Wallpaper};


pub fn find_non_fitting(state: &mut State, paths: Vec<String>, display_id: Option<u32>, is_verbose: bool) -> ActionResult {
    let (wallpapers, mut total_warning) = if paths.is_empty() {
        state.find_all_child_wallpapers(state.nodes.values())
    } else {
        let nodes = paths.iter()
            .map(|path| state.get_or_insert_node(path))
            .collect::<Vec<SharedWallpaperNode>>();

        state.find_all_child_wallpapers(nodes)
    };

    find_non_fitting_of(wallpapers, display_id, is_verbose).map(|success| {
        total_warning.append(success.warning.message());
        ActionSuccess {
            message: success.message,
            warning: total_warning,
        }
    })
}

fn find_non_fitting_of(wallpapers: IndexSet<Wallpaper>, display_id: Option<u32>, is_verbose: bool) -> ActionResult {

    let display_resol = get_display_resolution(display_id)?;
    let mut message = String::new();

    if is_verbose {
        let _ = writeln!(message, "Display: {}, {}", display_resol, display_resol.ratio_str());
    }

    let (info_map, warning) = get_media_info(wallpapers);

    for (path, info) in info_map {
        let resol = info.resolution;

        if info.is_video {
            if resol != display_resol {
                if is_verbose {
                    let _ = writeln!(message, "{}, {}, {}", resol, resol.ratio_str(), path);
                } else {
                    let _ = writeln!(message, "{path}");
                }
            }
        } else {
            if info.mode.is_default() && resol.ratio_equals(&display_resol) {
                if is_verbose {
                    let _ = writeln!(message, "{}, {}, {}", resol, resol.ratio_str(), path);
                } else {
                    let _ = writeln!(message, "{path}");
                }
            }
        }
    }

    Ok(ActionSuccess { message, warning })
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
            )),
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

struct MediaInfo {
    pub resolution: Resolution,
    pub is_video: bool,
    pub mode: DisplayMode,
}

fn get_media_info<I>(wallpapers: I) -> (HashMap<String, MediaInfo>, Warning)
where
    I: IntoIterator<Item = Wallpaper, IntoIter: ExactSizeIterator>,
{
    let iter = wallpapers.into_iter();
    let mut info_map = HashMap::with_capacity(iter.len());
    let mut rest = Vec::new();
    let mut warning = Warning::new();

    for wallpaper in iter {
        let path = wallpaper.path();

        match imagesize::size(path) {
            Ok(size) => {
                let Wallpaper { path, mode } = wallpaper;

                let prev = info_map.insert(
                    path,
                    MediaInfo {
                        resolution: Resolution::new(size.width as u64, size.height as u64),
                        is_video: false,
                        mode,
                    },
                );

                assert!(prev.is_none());
            }

            Err(ImageError::CorruptedImage) => warning.append_msg_path("Image corrupted", path),

            _ => rest.push(wallpaper),
        }
    }

    for wallpaper in rest {
        let path = wallpaper.path();

        match ffprobe::ffprobe(&path) {
            Ok(ffprobe) => {
                let stream = ffprobe
                    .streams
                    .iter()
                    .find(|stream| is_codec_type_video(&stream.codec_type));

                match stream {
                    Some(stream) => {
                        let Wallpaper { path, mode } = wallpaper;

                        let file_info = match (stream.width, stream.height) {
                            (Some(width), Some(height)) => MediaInfo {
                                resolution: Resolution::new(width as u64, height as u64),
                                is_video: true,
                                mode,
                            },

                            _ => {
                                warning.append_msg_path("Unable to get video resolution", &path);
                                continue;
                            }
                        };

                        let prev = info_map.insert(path, file_info);
                        assert!(prev.is_none());
                    }

                    None => warning.append_msg_path("File doesn't contain video stream", path),
                }
            }

            Err(FfProbeError::Io(err)) => {
                warning.append_msg_error_path("ffprobe I/O error", &err, path)
            }
            Err(err) => warning.append_msg_error_path("ffprobe error", &err, path),
        };
    }

    (info_map, warning)
}

fn is_codec_type_video(codec_type: &Option<String>) -> bool {
    codec_type
        .as_ref()
        .is_some_and(|codec_type| codec_type == "video")
}
