use crate::server::action::Action::*;
use crate::server::state::State;
use crate::server::display_mode::DisplayMode;
use crate::server::args_parse_error::ArgsParseError;
use crate::server::action_perform_error::ActionPerformError;
use crate::args_parse_error_format;
use crate::GET_HELP_MESSAGE;


macro_rules! add_arg_to_single {
    ($var:expr, $arg:expr) => {
        if $var.is_empty() {
            *($var) = $arg.clone();
        } else {
            return Err(args_parse_error_format!("Unrecognized argument: '{}'", $arg));
        }
    };
}

macro_rules! add_arg_to_multiple {
    ($single:expr, $multiple:expr, $arg:expr) => {
        if $single.is_empty() {
            *($single) = $arg.clone();
        } else {
            $multiple.push($arg.clone());
        }
    };
}


#[derive(PartialEq)]
pub enum Action {
    None,
    Help,

    GetWallpaper,
    SetWallpaper { path: String, mode: Option<DisplayMode> },
    Configure    { path: String, mode: Option<DisplayMode> },
    SetRandowWallpaper,

    GetGroupList,
    NewGroup        { name: String, paths: Vec<String> },
    GetGroup        { name: String },
    SetGroup        { name: String, mode: Option<DisplayMode> },
    AddToGroup      { name: String, paths: Vec<String> },
    RemoveFromGroup { name: String, paths: Vec<String> },
    ClearGroup      { name: String },
    RemoveGroup     { name: String },
}

impl Action {

    pub fn from_literal(literal: &str, cmd: &str) -> Result<Self, ArgsParseError> {
        match literal {
            "help"              => Ok(Help),
            "get"               => Ok(GetWallpaper),
            "set"               => Ok(SetWallpaper { path: String::new(), mode: Option::None }),
            "cofigure"          => Ok(Configure    { path: String::new(), mode: Option::None }),
            "random"            => Ok(SetRandowWallpaper),
            "list-groups"       => Ok(GetGroupList),
            "new-group"         => Ok(NewGroup        { name: String::new(), paths: Vec::new() }),
            "get-group"         => Ok(GetGroup        { name: String::new() }),
            "set-group"         => Ok(SetGroup        { name: String::new(), mode: Option::None }),
            "add-to-group"      => Ok(AddToGroup      { name: String::new(), paths: Vec::new() }),
            "remove-from-group" => Ok(RemoveFromGroup { name: String::new(), paths: Vec::new() }),
            "clear-group"       => Ok(ClearGroup      { name: String::new() }),
            "remove-group"      => Ok(RemoveGroup     { name: String::new() }),

            _ => return Err(args_parse_error_format!("Unrecognized action: '{literal}'. Use '{cmd} --help' to get more information"))
        }
    }


    pub fn get_name(&self) -> &str {
        match self {
            None               { .. } => "none",
            Help               { .. } => "help",
            GetWallpaper       { .. } => "get",
            SetWallpaper       { .. } => "set",
            Configure          { .. } => "cofigure",
            SetRandowWallpaper { .. } => "random",
            GetGroupList       { .. } => "list-groups",
            NewGroup           { .. } => "new-group",
            GetGroup           { .. } => "get-group",
            SetGroup           { .. } => "set-group",
            AddToGroup         { .. } => "add-to-group",
            RemoveFromGroup    { .. } => "remove-from-group",
            ClearGroup         { .. } => "clear-group",
            RemoveGroup        { .. } => "remove-group",
        }
    }


    pub fn add_arg(&mut self, arg: &String) -> Result<(), ArgsParseError> {
        match self {
            SetWallpaper { path, .. } => add_arg_to_single!(path, arg),
            Configure    { path, .. } => add_arg_to_single!(path, arg),
            SetGroup     { name, .. } => add_arg_to_single!(name, arg),
            GetGroup     { name }     => add_arg_to_single!(name, arg),
            ClearGroup   { name }     => add_arg_to_single!(name, arg),
            RemoveGroup  { name }     => add_arg_to_single!(name, arg),

            AddToGroup      { name, paths } => add_arg_to_multiple!(name, paths, arg),
            RemoveFromGroup { name, paths } => add_arg_to_multiple!(name, paths, arg),

            _ => return Err(args_parse_error_format!("Can't add arguments for '{}' action", self.get_name()))
        }

        Ok(())
    }


    pub fn perform(self, cmd: &str, state: &mut State) -> Result<String, ActionPerformError> {
        match self {
            None                    => Err(ActionPerformError::new(format!("No action specified. Use '{cmd} --help' to get more information"))),
            Help                    => Ok(GET_HELP_MESSAGE!(cmd)),
            GetWallpaper            => Ok(state.clone_current_wallpaper_path().unwrap_or_default()),
            SetWallpaper     { .. } => todo!(),
            Configure        { path, mode } => state.configure_wallpaper(path, mode),
            SetRandowWallpaper      => state.set_random_wallpaper(),
            GetGroupList            => todo!(),
            NewGroup         { .. } => todo!(),
            GetGroup         { .. } => todo!(),
            SetGroup         { .. } => todo!(),
            AddToGroup       { .. } => todo!(),
            RemoveFromGroup  { .. } => todo!(),
            ClearGroup       { .. } => todo!(),
            RemoveGroup      { .. } => todo!(),
        }
    }
}