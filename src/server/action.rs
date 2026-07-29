use crate::server::action::Action::*;
use crate::server::settings::Settings;
use crate::server::state::State;
use crate::server::args_parse_error::ArgsParseError;
use crate::server::action_perform_error::ActionPerformError;
use crate::args_parse_error_format;
use crate::GET_HELP_MESSAGE;


#[derive(PartialEq)]
pub enum Action {
    None,
    Help,

    GetWallpaperList,
    GetWallpaper,
    SetWallpaper { path: String, settings: Settings },
    SetRandowWallpaper,
    AddNodes     { paths: Vec<String>, settings: Settings },
    RemoveNodes  { paths: Vec<String> },

    GetGroupList,
    NewGroup        { name: String, paths: Vec<String> },
    GetGroup        { name: String },
    SetGroup        { name: String, settings: Settings },
    AddToGroup      { name: String, paths: Vec<String> },
    RemoveFromGroup { name: String, paths: Vec<String> },
    ClearGroup      { name: String },
    RemoveGroup     { name: String },
}

impl Action {

    pub fn from_literal(literal: &str, cmd: &str) -> Result<Self, ArgsParseError> {
        match literal {
            "help"              => Ok(Help),
            "list"              => Ok(GetWallpaperList),
            "get"               => Ok(GetWallpaper),
            "set"               => Ok(SetWallpaper { path: String::new(), settings: Settings::new() }),
            "random"            => Ok(SetRandowWallpaper),
            "add"               => Ok(AddNodes     { paths: Vec::new(), settings: Settings::new() }),
            "remove"            => Ok(RemoveNodes  { paths: Vec::new() }),
            "group-list"        => Ok(GetGroupList),
            "new-group"         => Ok(NewGroup        { name: String::new(), paths: Vec::new() }),
            "get-group"         => Ok(GetGroup        { name: String::new() }),
            "set-group"         => Ok(SetGroup        { name: String::new(), settings: Settings::new() }),
            "add-to-group"      => Ok(AddToGroup      { name: String::new(), paths: Vec::new() }),
            "remove-from-group" => Ok(RemoveFromGroup { name: String::new(), paths: Vec::new() }),
            "clear-group"       => Ok(ClearGroup      { name: String::new() }),
            "remove-group"      => Ok(RemoveGroup     { name: String::new() }),

            _ => Err(args_parse_error_format!("Unrecognized action: '{literal}'. Use '{cmd} --help' to get more information"))
        }
    }


    pub fn get_name(&self) -> &str {
        match self {
            None               { .. } => "none",
            Help               { .. } => "help",
            GetWallpaperList   { .. } => "list",
            GetWallpaper       { .. } => "get",
            SetWallpaper       { .. } => "set",
            SetRandowWallpaper { .. } => "random",
            AddNodes           { .. } => "add",
            RemoveNodes        { .. } => "remove",
            GetGroupList       { .. } => "group-list",
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
            Help => {},

            AddNodes     { paths, .. } |
            RemoveNodes  { paths, .. } => {
                paths.push(arg.clone());
            },

            SetWallpaper { path: name, .. } |
            SetGroup    { name, .. } |
            GetGroup    { name, .. } |
            ClearGroup  { name, .. } |
            RemoveGroup { name, .. } => {
                if name.is_empty() {
                    *name = arg.clone();
                } else {
                    return Err(args_parse_error_format!("Unrecognized argument: '{arg}'"));
                }
            },

            NewGroup        { name, paths } |
            AddToGroup      { name, paths } |
            RemoveFromGroup { name, paths } => {
                if name.is_empty() {
                    *name = arg.clone();
                } else {
                    paths.push(arg.clone());
                }
            },

            _ => return Err(args_parse_error_format!("Can't add arguments for '{}' action", self.get_name()))
        }

        Ok(())
    }


    pub fn add_setting<F>(&mut self, arg: &str, next_arg: Option<&String>, func: F) -> Result<(), ArgsParseError>
    where
        F: Fn(&mut Settings, &String) -> Result<(), ArgsParseError>
    {
        match self {
            Help => Ok(()),

            SetWallpaper { settings, .. } |
            AddNodes     { settings, .. } |
            SetGroup     { settings, .. } => {
                match next_arg {
                    Option::Some(value) => func(settings, value)?,
                    Option::None => return Err(args_parse_error_format!("Expected value for '{arg}', got end of arguments"))
                };

                Ok(())
            }

            None => Err(args_parse_error_format!("Cannot use '{}' before action", arg)),
            _    => Err(args_parse_error_format!("Cannot use '{}' for action {}", arg, self.get_name())),
        }
    }


    pub fn validate(&self) -> Result<(), ArgsParseError> {
        match self {
            NewGroup    { name, .. } |
            GetGroup    { name, .. } |
            SetGroup    { name, .. } |
            ClearGroup  { name, .. } |
            RemoveGroup { name, .. } => {
                self.validate_name(name)?;
            },

            AddNodes    { paths, .. } |
            RemoveNodes { paths, .. } => {
                self.validate_paths(paths)?;
            }

            AddToGroup      { name, paths } |
            RemoveFromGroup { name, paths } => {
                self.validate_name(name)?;
                self.validate_paths(paths)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn validate_name(&self, name: &str) -> Result<(), ArgsParseError> {
        if name.is_empty() {
            return Err(args_parse_error_format!("Action '{}' requires not empty name", self.get_name()));
        }
        
        Ok(())
    }

    fn validate_paths(&self, paths: &Vec<String>) -> Result<(), ArgsParseError> {
        if paths.is_empty() {
            return Err(args_parse_error_format!("Action '{}' requires at least one path", self.get_name()));
        }

        Ok(())
    }


    pub fn perform(self, cmd: &str, state: &mut State) -> Result<String, ActionPerformError> {
        match self {
            None => return Err(ActionPerformError::new(format!("No action specified. Use '{cmd} --help' to get more information"))),
            Help => return Ok(GET_HELP_MESSAGE!(cmd)),

            GetWallpaperList => return Ok(state.get_wallpaper_list()),

            GetWallpaper => return state
                    .get_current_wallpaper_path().clone()
                    .ok_or_else(|| ActionPerformError::new("No wallpaper is currently set")),

            SetWallpaper { path, settings } => state.set_wallpaper(path, &settings)?,
            SetRandowWallpaper              => return state.set_random_wallpaper(),

            AddNodes     { paths, settings } => state.add_nodes(&paths, &settings)?,
            RemoveNodes  { paths }           => state.remove_nodes(&paths),

            GetGroupList                        => return Ok(state.get_group_list()),
            GetGroup         { name }           => return state.get_group_info(&name),
            NewGroup         { name, paths }    => state.new_group(name, &paths)?,
            SetGroup         { name, settings } => return state.set_group(&name, &settings),
            AddToGroup       { name, paths }    => state.add_to_group(name, &paths)?,
            RemoveFromGroup  { name, paths }    => state.remove_from_group(&name, &paths)?,
            ClearGroup       { name }           => state.clear_group(&name)?,
            RemoveGroup      { name }           => state.remove_group(&name),
        }

        Ok(String::new())
    }
}