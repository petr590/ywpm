use crate::{action_perform_error_localized, format_localized};
use crate::cli::{ParsedTimePeriod, Settings};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess};
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

        let (wallpapers, warning) = state.find_all_child_wallpapers([&node]);
        eprint!("{warning}");

        state.run_backend_with_random_wallpaper(&wallpapers)?;
    }

    Ok(())
}

pub fn get_current(state: &State, is_verbose: bool) -> ActionResult {
    let path = state.current_wallpaper_path
        .clone()
        .ok_or_else(|| action_perform_error_localized!(
            "No wallpaper is set",
            "Обои не установлены"
        ))?;
    
    if is_verbose {
        todo!("get image size")
    } else {
        Ok(ActionSuccess::with_message(path))
    }
}

pub fn set(state: &mut State, path: impl Into<String>, settings: &Settings, period: ParsedTimePeriod) -> ActionResult {
    let path = path.into();
    let node = state.update_node(&path, settings, period.clone());

    if TimePeriod::is_none_or_now(&node.borrow().period) {
        set_node(state, [&node])
    } else {
        Ok(format_localized!(
            "The wallpaper is scheduled for the {}: '{}'",
            "Установка обоев запланирована на период {}: '{}'",
            period.unwrap(),
            path
        ).into())
    }
}

pub fn set_random(state: &mut State) -> ActionResult {
    set_node(state, state.find_actual_nodes().iter())
}

pub fn reset(state: &mut State) {
    backend::stop();
    state.current_wallpaper_path = None;
}

fn set_node<'a>(state: &mut State, nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>) -> ActionResult {

    let (wallpapers, warning) = state.find_all_child_wallpapers(nodes);
    let wallpaper = state.run_backend_with_random_wallpaper(&wallpapers)?;

    Ok(format_localized!(
        "{}Wallpaper set: '{}'",
        "{}Установлены обои: '{}'",
        warning, wallpaper.path()
    ).into())
}
