use crate::cli::{ActionSubcommand::{self, *}, Cli};
use crate::core::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::service::{config, group, media, node, wallpaper};
use crate::state::State;
use crate::util;

pub fn perform_action_and_update_config(cli: Cli, state: &mut State) -> ActionResult {
    let is_verbose = cli.is_verbose();

    let needs_update = needs_config_update(cli.subcommand());

    perform(cli.subcommand_move(), state, is_verbose).and_then(|msg| {
        if needs_update {
            config::write(state, util::get_config_path())
                .map_err(ActionPerformError::from_boxed)?;
        }

        Ok(msg)
    })
}

fn perform(subcommand: ActionSubcommand, state: &mut State, is_verbose: bool) -> ActionResult {
    match subcommand {
        GetCurrentWallpaper                     => return wallpaper::get_current(state, is_verbose),
        SetWallpaper { path, settings, period } => return wallpaper::set(state, path, &settings, period),
        SetRandowWallpaper                      => return wallpaper::set_random(state),
        ResetWallpaper                          => wallpaper::reset(state),
        RestoreWallpaper                        => wallpaper::restore(state)?,

        GetNodeList                          => return Ok(node::get_list(state).into()),
        AddNodes { paths, settings, period } => node::add(state, &paths, &settings, period)?,
        RemoveNodes { paths }                => node::remove(state, &paths),
        ClearNodes                           => node::clear(state),

        GetGroupList                               => return Ok(group::get_list(state).into()),
        GetGroup        { name }                   => return group::get_info(state, &name),
        NewGroup        { name, paths }            => group::new(state, name, &paths)?,
        SetGroup        { name, settings, period } => return group::set(state, &name, &settings, period),
        AddToGroup      { name, paths }            => group::add_to_group(state, name, &paths)?,
        RemoveFromGroup { name, paths }            => group::remove_from_group(state, &name, &paths)?,
        ClearGroup      { name }                   => group::clear(state, &name)?,
        RemoveGroup     { name }                   => group::remove(state, &name),

        FindNonFittingWallpapers { paths, display_id } => {
            return media::find_non_fitting(state, paths, display_id, is_verbose);
        },
    }

    Ok(ActionSuccess::new())
}

fn needs_config_update(subcommand: &ActionSubcommand) -> bool {
    match subcommand {
        SetWallpaper             { .. } |
        SetRandowWallpaper       { .. } |
        ResetWallpaper           { .. } |
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