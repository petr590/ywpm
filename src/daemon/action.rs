use crate::daemon::find_non_fitting_wallpapers;
use crate::{GET_HELP_MESSAGE, action_perform_error_localized};
use crate::daemon::time_period::TimePeriod;
use crate::daemon::action_perform_error::ActionPerformError;
use crate::daemon::settings::Settings;
use crate::daemon::state::State;

use self::Action::*;

#[derive(Debug, PartialEq)]
pub enum Action {
    Help,

    GetWallpaperList,
    GetWallpaper,
    SetWallpaper { path: String, settings: Settings, period: Option<TimePeriod> },
    SetRandowWallpaper,
    
    AddNodes    { paths: Vec<String>, settings: Settings },
    RemoveNodes { paths: Vec<String> },

    GetGroupList,
    NewGroup        { name: String, paths: Vec<String> },
    GetGroup        { name: String },
    SetGroup        { name: String, settings: Settings, period: Option<TimePeriod> },
    AddToGroup      { name: String, paths: Vec<String> },
    RemoveFromGroup { name: String, paths: Vec<String> },
    ClearGroup      { name: String },
    RemoveGroup     { name: String },

    FindNonFittingWallpapers { paths: Vec<String>, display_id: Option<u32> },
}

impl Action {
    
    pub fn perform(self, cmd: &str, state: &mut State) -> Result<String, ActionPerformError> {
        match self {
            Help => return Ok(GET_HELP_MESSAGE!(cmd)),

            GetWallpaperList => return Ok(state.get_wallpaper_list()),

            GetWallpaper => return state
                    .get_current_wallpaper_path().clone()
                    .ok_or_else(|| action_perform_error_localized!(
                        "No wallpaper is set",
                        "Обои не установлены"
                    )),

            SetWallpaper { path, settings, period } => return state.set_wallpaper(path, &settings, period),
            SetRandowWallpaper                      => return state.set_random_wallpaper(),

            AddNodes     { paths, settings } => state.add_nodes(&paths, &settings)?,
            RemoveNodes  { paths }           => state.remove_nodes(&paths),

            GetGroupList                                => return Ok(state.get_group_list()),
            GetGroup         { name }                   => return state.get_group_info(&name),
            NewGroup         { name, paths }            => state.new_group(name, &paths)?,
            SetGroup         { name, settings, period } => return state.set_group(&name, &settings, period),
            AddToGroup       { name, paths }            => state.add_to_group(name, &paths)?,
            RemoveFromGroup  { name, paths }            => state.remove_from_group(&name, &paths)?,
            ClearGroup       { name }                   => state.clear_group(&name)?,
            RemoveGroup      { name }                   => state.remove_group(&name),

            FindNonFittingWallpapers { paths, display_id } => return find_non_fitting_wallpapers::run(paths, display_id),
        }

        Ok(String::new())
    }
}