use std::collections::HashMap;
use std::fmt::Write;

use comfy_table::{ContentArrangement, Table, presets};
use display_info::DisplayInfo;
use imagesize::{ImageError, ImageResult};
use indexmap::IndexSet;
use itertools::Itertools;

use crate::{action_perform_error_localized, format_localized, localized, util};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::service::resolution::Resolution;
use crate::core::Warning;
use crate::state::{DisplayMode, SharedWallpaperNode, State, Wallpaper};


struct Row {
    path: String,
    resolution: Resolution,
    diff_percent: u64,
    is_wider: bool,
}


pub fn find_non_fitting(state: &mut State, paths: Vec<String>, display_id: Option<u32>, is_verbose: bool, term_width: u16) -> ActionResult {
    let display_res = get_display_resolution(display_id)?;

    let (wallpapers, mut warning) = get_wallpapers(state, paths);
    let (info_map, warning2)      = get_media_info_for_all(wallpapers);

    warning.append(warning2.message());


    let mut message = String::new();

    if is_verbose {
        let mut non_fitting = Vec::new();
        
        find_non_fitting_of(&display_res, info_map,
            |path, info| {
                let resolution = info.resolution;
                let (diff_percent, is_wider) = resolution.calculate_difference_in_percent(&display_res);
                non_fitting.push(Row { path, resolution, diff_percent, is_wider });
            }
        );

        non_fitting.sort_by(|row1, row2| {
            row1.is_wider.cmp(&row2.is_wider)
                .then_with(|| row1.diff_percent.cmp(&row2.diff_percent))
                .then_with(|| row1.path.cmp(&row2.path))
        });


        let mut table = Table::new();

        table
            .load_style(presets::UTF8_FULL)
            .set_width(term_width)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(localized!(
                ["Resolution", "Ratio",       "Difference", "Path"],
                ["Разрешение", "Соотношение", "Разница",    "Путь"],
            ));
        

        let home = util::get_home_with_slash();

        for Row { path, resolution, diff_percent, is_wider } in non_fitting {
            table.add_row([
                resolution.to_string(),
                resolution.ratio_str(),
                diff_to_str(diff_percent, is_wider),
                util::replace_home_with_tilde(&home, &path)
            ]);
        }

        let _ = writeln!(message, "{table}");
        let _ = writeln!(message, "Display: {}, {}", display_res, display_res.ratio_str());

    } else {
        find_non_fitting_of(&display_res, info_map,
            |path, _| {
                let _ = writeln!(message, "{path}");
            }
        );
    }
    
    Ok(ActionSuccess { message, warning })
}


fn find_non_fitting_of(display_res: &Resolution, info_map: HashMap<String, MediaInfo>, mut consumer: impl FnMut(String, MediaInfo)) {

    for (path, info) in info_map {
        if info.is_video {
            if info.resolution != *display_res {
                consumer(path, info)
            }

        } else {
            if  info.mode.is_default() &&
                !info.resolution.ratio_equals(display_res)
            {
                consumer(path, info)
            }
        };
    }
}


fn diff_to_str(diff_percent: u64, is_wider: bool) -> String {
    let diff_str = if diff_percent > 0 {
        diff_percent.to_string()
    } else {
        String::from("<1")
    };

    if is_wider {
        format_localized!("wider by {diff_str}%", "шире на {diff_str}%")
    } else {
        format_localized!("taller by {diff_str}%", "выше на {diff_str}%")
    }
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

fn get_wallpapers(state: &mut State, paths: Vec<String>) -> (IndexSet<Wallpaper>, Warning) {
    if paths.is_empty() {
        state.find_all_child_wallpapers(state.nodes.values())
    } else {
        let nodes = paths.iter()
            .map(|path| state.get_or_insert_node(path))
            .collect::<Vec<SharedWallpaperNode>>();

        state.find_all_child_wallpapers(nodes)
    }
}

struct MediaInfo {
    pub resolution: Resolution,
    pub is_video: bool,
    pub mode: DisplayMode,
}

fn get_media_info_for_all<I>(wallpapers: I) -> (HashMap<String, MediaInfo>, Warning)
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
                let prev = info_map.insert(wallpaper.path, MediaInfo {
                    resolution,
                    is_video: false,
                    mode: wallpaper.mode,
                });

                assert!(prev.is_none());
            }

            Err(ImageError::CorruptedImage) => warning.append_msg_path("Image corrupted", path),
            Err(_) => rest.push(wallpaper),
        }
    }

    for wallpaper in rest {
        match get_video_resolution(wallpaper.path()) {
            Ok(resolution) => {
                let prev = info_map.insert(wallpaper.path, MediaInfo {
                    resolution,
                    is_video: true,
                    mode: wallpaper.mode
                });

                assert!(prev.is_none());
            }

            Err(warn) => warning.append(warn.message()),
        }
    }

    (info_map, warning)
}


pub(super) fn get_resolution(path: &str) -> Result<Resolution, Warning> {
    match get_image_resolution(path) {
        Ok(resolution) => return Ok(resolution),
        Err(ImageError::CorruptedImage) => return Err(Warning::with_msg_path("Image corrupted", path)),
        Err(_) => {}
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