use std::fmt::Write;
use std::path::PathBuf;

use crate::daemon::service::media;
use crate::{action_perform_error_localized, format_localized, writeln_localized};
use crate::cli::{ParsedTimePeriod, Settings};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess, Warning};
use crate::daemon::backend;
use crate::state::{SharedWallpaperNode, State, TimePeriod};

pub fn restore(state: &mut State) -> Result<(), ActionPerformError> {
    if backend::is_running() {
        return Err(action_perform_error_localized!(
            "Wallpaper already restored",
            "Обои уже восстановлены"
        ));
    }

    if let Some(current_path) = &state.current_wallpaper_path {

        let node = match state.find_closest_node(current_path) {
            Some(node) => {
                let node = node.borrow();
                SharedWallpaperNode::new(
                    current_path,
                    node.mode.clone(),
                    node.recursive_level,
                    node.period.clone(),
                )
            }

            None => state.get_or_insert_node(&current_path.clone()),
        };

        let mut warning = Warning::new();
        let wallpapers = state.find_all_child_wallpapers([&node], &mut warning);
        eprint!("{warning}");

        state.run_backend_with_random_wallpaper(&wallpapers)?;
    }

    Ok(())
}

pub fn current_wallpaper_path_or_error(state: &State) -> Result<PathBuf, ActionPerformError> {
    state.current_wallpaper_path
        .clone()
        .ok_or_else(|| action_perform_error_localized!(
            "No wallpaper is set",
            "Обои не установлены"
        ))
}

pub fn get_current(state: &State, is_verbose: bool, warning: &mut Warning) -> ActionResult {
    let path = current_wallpaper_path_or_error(state)?;
    
    if is_verbose {
        let mut message = String::new();

        let _ = writeln_localized!(
            message, 
            "Path: {}",
            "Путь: {}",
            path.display()
        );

        if let Some(node) = state.find_closest_node(&path) {
            let _ = writeln_localized!(
                message,
                "Mode: {}",
                "Режим: {}",
                node.borrow().mode
            );

            let _ = writeln_localized!(
                message,
                "Period: {}",
                "Период: {}",
                TimePeriod::opt_to_string(&node.borrow().period)
            );
        }

        match media::get_resolution(&path) {
            Ok(resolution) => {
                let _ = writeln_localized!(
                    message,
                    "Resolution: {}, {}",
                    "Разрешение: {}, {}",
                    resolution, resolution.ratio_str()
                );
            }

            Err(warn) => {
                warning.append(warn.message());
            }
        }

        Ok(ActionSuccess::with_message(message))

    } else {
        Ok(ActionSuccess::with_message(path.to_string_lossy()))
    }
}

pub fn set(state: &mut State, path: impl Into<PathBuf>, settings: &Settings, period: ParsedTimePeriod, warning: &mut Warning) -> ActionResult {
    let path = path.into();
    let node = state.update_node(&path, settings, period.clone());

    if TimePeriod::is_none_or_now(&node.borrow().period) {
        set_one_of_nodes(state, [&node], warning)

    } else if let ParsedTimePeriod::Set(period) = period {
        Ok(format_localized!(
            "The wallpaper is scheduled for the {}: '{}'",
            "Установка обоев запланирована на период {}: '{}'",
            period, path.display()
        ).into())

    } else {
        Ok(ActionSuccess::new())
    }
}

pub fn set_random(state: &mut State, warning: &mut Warning) -> ActionResult {
    set_one_of_nodes(state, state.find_actual_nodes().iter(), warning)
}

pub fn reset(state: &mut State) {
    backend::stop();
    state.current_wallpaper_path = None;
}

fn set_one_of_nodes<'a>(state: &mut State, nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>, warning: &mut Warning) -> ActionResult {

    let wallpapers = state.find_all_child_wallpapers(nodes, warning);
    let wallpaper = state.run_backend_with_random_wallpaper(&wallpapers)?;

    Ok(format_localized!(
        "Wallpaper set: '{}'",
        "Установлены обои: '{}'",
        wallpaper.path().display()
    ).into())
}
