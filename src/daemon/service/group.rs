use std::collections::HashSet;

use itertools::Itertools;

use crate::daemon::action::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::arg_parsing::{ParsedTimePeriod, Settings};
use crate::daemon::state::{SharedWallpaperNode, State, TimePeriod, WallpaperGroup};
use crate::{action_perform_error_localized, format_localized};

pub fn get_list(state: &State) -> String {
    if state.groups.is_empty() {
        format_localized!(
            "No groups are found",
            "Ни одна группа не найдена"
        )

    } else {
        format_localized!(
            "Groups: ['{}']",
            "Группы: ['{}']",
            state.groups.keys().format("', '")
        )
    }
}

pub fn get_info(state: &State, name: &str) -> ActionResult {
    match state.groups.get(name) {
        Some(group) => {
            yaml_serde::to_string(&group.as_dto())
                    .map(ActionSuccess::with_message)
                    .map_err(|err| ActionPerformError::new(err.to_string()))
        }

        None => Err(group_not_found_error(name))
    }
}

pub fn new(state: &mut State, name: impl Into<String>, paths: &Vec<String>) -> Result<(), ActionPerformError> {
    let group_nodes = paths.iter()
        .map(|path| state.get_or_insert_node(path))
        .collect::<HashSet<SharedWallpaperNode>>();

    state.groups
        .entry(name.into())
        .or_insert_with(WallpaperGroup::new)
        .add_all(group_nodes);

    Ok(())
}

pub fn set(state: &mut State, name: &str, settings: &Settings, period: ParsedTimePeriod) -> ActionResult {
    let group = get_group_mut(state, name)?;

    if settings.is_some() {
        for node in group.nodes() {
            settings.update_node(&mut node.borrow_mut());
        }
    }

    match period {
        ParsedTimePeriod::NotSpecified    => {}
        ParsedTimePeriod::Reset           => group.period = None,
        ParsedTimePeriod::Set(ref period) => group.period = Some(period.clone()),
    }

    let group = state.groups.get(name).unwrap();

    if TimePeriod::is_none_or_now(&group.period) {
        let (wallpapers, warning) = state.find_all_child_wallpapers(group.nodes());
        let wallpaper = state.run_backend_with_random_wallpaper(&wallpapers)?;

        Ok(format_localized!(
            "{}Wallpaper group set: '{}'",
            "{}Установлена группа: '{}'",
            warning,
            wallpaper.path()
        ).into())

    } else {
        Ok(format_localized!(
            "The '{}' wallpaper group is scheduled for the {}",
            "Установка группы '{}' запланирована на период {}",
            name,
            period.unwrap()
        ).into())
    }
}

pub fn remove(state: &mut State, name: &str) {
    state.groups.remove(name);
}

pub fn add_to_group(state: &mut State, name: impl Into<String>, paths: &Vec<String>) -> Result<(), ActionPerformError> {
    new(state, name, paths)
}

pub fn remove_from_group(state: &mut State, name: &str, paths: &Vec<String>) -> Result<(), ActionPerformError> {
    let group = get_group_mut(state, name)?;

    for path in paths {
        group.remove(&path);
    }

    Ok(())
}

pub fn clear(state: &mut State, name: &str) -> Result<(), ActionPerformError> {
    get_group_mut(state, name)?.clear();
    Ok(())
}

fn get_group_mut<'a>(state: &'a mut State, name: &str) -> Result<&'a mut WallpaperGroup, ActionPerformError> {
    state.groups
        .get_mut(name)
        .ok_or_else(|| group_not_found_error(name))
}

fn group_not_found_error(name: &str) -> ActionPerformError {
    action_perform_error_localized!(
        "Group '{name}' not found",
        "Группа '{name}' не найдена"
    )
}
