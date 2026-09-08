use crate::daemon::action::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::arg_parsing::ParsedTimePeriod;
use crate::daemon::service;
use crate::daemon::state::{Settings, State};
use crate::{GET_HELP_MESSAGE, action_perform_error_localized, util};

use self::Action::*;

#[derive(Debug, PartialEq)]
pub enum Action {
    Help,

    GetCurrentWallpaper,
    SetWallpaper { path: String, settings: Settings, period: ParsedTimePeriod },
    SetRandowWallpaper,

    GetNodeList,
    AddNodes    { paths: Vec<String>, settings: Settings },
    RemoveNodes { paths: Vec<String> },
    ClearNodes,

    GetGroupList,
    GetGroup        { name: String },
    NewGroup        { name: String, paths: Vec<String> },
    SetGroup        { name: String, settings: Settings, period: ParsedTimePeriod },
    AddToGroup      { name: String, paths: Vec<String> },
    RemoveFromGroup { name: String, paths: Vec<String> },
    ClearGroup      { name: String },
    RemoveGroup     { name: String },

    FindNonFittingWallpapers { paths: Vec<String>, display_id: Option<u32>, is_short: bool },
}

impl Action {
    pub fn perform_and_update_config(self, cmd: &str, state: &mut State) -> ActionResult {
        let needs_update = self.needs_config_update();

        self.perform(cmd, state).and_then(|msg| {
            if needs_update {
                service::config::write(state, util::get_config_path())
                    .map_err(ActionPerformError::from_boxed)?;
            }

            Ok(msg)
        })
    }

    fn perform(self, cmd: &str, state: &mut State) -> ActionResult {
        match self {
            Help => return Ok(GET_HELP_MESSAGE!(cmd).into()),

            GetCurrentWallpaper => {
                return state.current_wallpaper_path
                    .clone()
                    .map(ActionSuccess::with_message)
                    .ok_or_else(|| action_perform_error_localized!(
                        "No wallpaper is set",
                        "Обои не установлены"
                    ));
            }

            SetWallpaper { path, settings, period } => return service::wallpaper::set(state, path, &settings, period),
            SetRandowWallpaper                      => return service::wallpaper::set_random(state),

            GetNodeList                  => return Ok(service::node::get_list(state).into()),
            AddNodes { paths, settings } => service::node::add(state, &paths, &settings)?,
            RemoveNodes { paths }        => service::node::remove(state, &paths),
            ClearNodes                   => service::node::clear(state),

            GetGroupList                               => return Ok(service::group::get_list(state).into()),
            GetGroup        { name }                   => return service::group::get_info(state, &name),
            NewGroup        { name, paths }            => service::group::new(state, name, &paths)?,
            SetGroup        { name, settings, period } => return service::group::set(state, &name, &settings, period),
            AddToGroup      { name, paths }            => service::group::add_to_group(state, name, &paths)?,
            RemoveFromGroup { name, paths }            => service::group::remove_from_group(state, &name, &paths)?,
            ClearGroup      { name }                   => service::group::clear(state, &name)?,
            RemoveGroup     { name }                   => service::group::remove(state, &name),

            FindNonFittingWallpapers { paths, display_id, is_short } => {
                return service::find_non_fitting_wallpapers::run(state, paths, display_id, is_short);
            },
        }

        Ok(ActionSuccess::new())
    }

    fn needs_config_update(&self) -> bool {
        match self {
            SetWallpaper             { .. } |
            SetRandowWallpaper       { .. } |
            AddNodes                 { .. } |
            RemoveNodes              { .. } |
            ClearNodes               { .. } |
            NewGroup                 { .. } |
            SetGroup                 { .. } |
            AddToGroup               { .. } |
            RemoveFromGroup          { .. } |
            ClearGroup               { .. } |
            RemoveGroup              { .. } |
            FindNonFittingWallpapers { .. } => true,

            _ => false,
        }
    }
}
