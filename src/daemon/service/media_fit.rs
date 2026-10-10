use std::collections::HashMap;
use std::fmt::Write;

use comfy_table::{ContentArrangement, Table, presets};
use display_info::DisplayInfo;
use indexmap::IndexSet;
use itertools::Itertools;

use crate::{action_perform_error_localized, format_localized, localized, util};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess, Warning};
use crate::daemon::service::media::{MediaInfo, MediaType};
use crate::daemon::service::resolution::Resolution;
use crate::state::{SharedWallpaperNode, State, Wallpaper};


struct Row {
    path: String,
    resolution: Resolution,
    diff_percent: u64,
    is_wider: bool,
}


pub fn find_non_fitting(state: &mut State, paths: Vec<String>, display_id: Option<u32>, is_verbose: bool, term_width: u16) -> ActionResult {
    let display_res = get_display_resolution(display_id)?;

    let (wallpapers, mut warning) = find_wallpapers_from_paths(state, paths);
    let (info_map, warning2)      = MediaInfo::get_for_all(wallpapers);

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
        match info.media_type {
            MediaType::Image => {
                if  info.mode.is_default() &&
                    !info.resolution.ratio_equals(display_res)
                {
                    consumer(path, info)
                }
            }

            MediaType::Video => {
                if info.resolution != *display_res {
                    consumer(path, info)
                }
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

fn find_wallpapers_from_paths(state: &mut State, paths: Vec<String>) -> (IndexSet<Wallpaper>, Warning) {
    if paths.is_empty() {
        state.find_all_child_wallpapers(state.nodes.values())
    } else {
        let nodes = paths.iter()
            .map(|path| state.get_or_insert_node(path))
            .collect::<Vec<SharedWallpaperNode>>();

        state.find_all_child_wallpapers(nodes)
    }
}
