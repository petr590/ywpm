use itertools::Itertools;

use crate::cli::{ParsedTimePeriod, Settings};
use crate::core::ActionPerformError;
use crate::daemon::backend;
use crate::state::{State, Wallpaper};

pub fn get_list(state: &State) -> String {
    if state.nodes.is_empty() {
        String::from("No wallpapers are found")
    } else {
        format!(
            "Wallpapers:\n{}",
            state.nodes.values()
                .map(|node| node.borrow().path().clone())
                .format("\n")
        )
    }
}

pub fn add(state: &mut State, paths: &Vec<String>, settings: &Settings, period: ParsedTimePeriod) -> Result<(), ActionPerformError> {
    let mut update_current = false;

    for path in paths {
        state.update_node(path, settings, period.clone());
        
        update_current = update_current ||
            state.current_wallpaper_path
                .as_ref()
                .is_some_and(|p| path.starts_with(p));
    }

    if update_current {
        let node = state.get_or_insert_node(&state.current_wallpaper_path.clone().unwrap());

        backend::run(&Wallpaper::from(&*node.borrow())).map_err(|err| {
            state.current_wallpaper_path = None;
            ActionPerformError::from_boxed(err)
        })?;
    }

    Ok(())
}

pub fn remove(state: &mut State, paths: &Vec<String>) {
    for path in paths {
        if state
            .current_wallpaper_path
            .as_ref()
            .is_some_and(|p| *p == *path)
        {
            backend::stop();
            state.current_wallpaper_path = None;
        }

        for (_, group) in &mut state.groups {
            group.remove(path);
        }

        state.nodes.remove(path);
    }
}

pub fn clear(state: &mut State) {
    state.current_wallpaper_path = None;
    state.nodes.clear();
    state.groups.clear();
}
