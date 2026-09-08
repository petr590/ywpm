use std::error::Error;

use crate::daemon::action::ActionResult;
use crate::daemon::arg_parsing::ParsedTimePeriod;
use crate::daemon::state::{Settings, SharedWallpaperNode, State, TimePeriod};
use crate::format_localized;

pub fn restore(state: &mut State) -> Result<(), Box<dyn Error>> {
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

fn set_node<'a>(state: &mut State, nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>) -> ActionResult {

    let (wallpapers, warning) = state.find_all_child_wallpapers(nodes);
    let wallpaper = state.run_backend_with_random_wallpaper(&wallpapers)?;

    Ok(format_localized!(
        "{}Wallpaper set: '{}'",
        "{}Установлены обои: '{}'",
        warning, wallpaper.path()
    ).into())
}
