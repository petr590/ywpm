use crate::cli::ActionSubcommand::{self, *};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess, Warning};
use crate::daemon::service::{state_io, group, media_fit, node, wallpaper};
use crate::state::State;

pub fn perform_action_and_update_config(subcommand: ActionSubcommand, state: &mut State, is_verbose: bool, term_width: u16, warning: &mut Warning) -> ActionResult {
    let needs_update = subcommand.changes_state();

    perform(subcommand, state, is_verbose, term_width, warning).and_then(|msg| {
        if needs_update {
            state_io::write(state.normalize())
                .map_err(ActionPerformError::from_boxed)?;
        }

        Ok(msg)
    })
}

fn perform(subcommand: ActionSubcommand, state: &mut State, is_verbose: bool, term_width: u16, warning: &mut Warning) -> ActionResult {
    match subcommand {
        GetCurrentWallpaper                     => return wallpaper::get_current(state, is_verbose, warning),
        SetWallpaper { path, settings, period } => return wallpaper::set(state, path, &settings, period, warning),
        SetRandowWallpaper                      => return wallpaper::set_random(state, warning),
        ResetWallpaper                          => wallpaper::reset(state),
        RestoreWallpaper                        => wallpaper::restore(state)?,

        GetNodeList => return Ok(node::get_list(state, is_verbose, term_width).into()),

        AddNodes    { paths, settings, period } => node::add_or_update(state, &paths, &settings, period)?,
        UpdateNodes { paths, settings, period } => node::update(state, paths, &settings, period)?,
        
        RemoveNodes { paths } => node::remove(state, &paths),
        ClearNodes            => node::clear(state),

        GetGroupList                               => return Ok(group::get_list(state).into()),
        GetGroup        { name }                   => return group::get_info(state, &name),
        NewGroup        { name, paths }            => group::new(state, name, &paths)?,
        SetGroup        { name, settings, period } => return group::set(state, &name, &settings, period, warning),
        AddToGroup      { name, paths }            => group::add_to_group(state, name, &paths)?,
        RemoveFromGroup { name, paths }            => group::remove_from_group(state, &name, &paths)?,
        ClearGroup      { name }                   => group::clear(state, &name)?,
        RemoveGroup     { name }                   => group::remove(state, &name),

        FindNonFittingWallpapers { paths, display_id } => {
            return media_fit::find_non_fitting(state, paths, display_id, is_verbose, term_width, warning);
        },
    }

    Ok(ActionSuccess::new())
}
