use std::collections::HashSet;

use crate::daemon::action::Action;
use crate::daemon::arg_parsing::time_period::ArgParseTimePeriod;
use crate::daemon::arg_parsing::{ArgParseError, ParsedTimePeriod::NotSpecified};
use crate::daemon::state::Settings;
use crate::{arg_parse_error_localized, arg_parse_error_localized_with_usage, util};

#[derive(PartialEq)]
pub(crate) enum UniqueOption {
    DisplayId(u32),
    ShortFlag,
}

#[derive(PartialEq)]
pub(crate) struct ArgParseAction {
    name: Option<String>,
    args: Vec<String>,

    settings_options: Vec<String>,
    period_options: Vec<String>,

    settings: Settings,
    period: ArgParseTimePeriod,

    unique_option_names: HashSet<String>,
    unique_options: Vec<UniqueOption>,
}

impl ArgParseAction {
    pub fn new() -> Self {
        Self {
            name:                None,
            args:                Vec::new(),
            settings_options:    Vec::new(),
            period_options:      Vec::new(),
            settings:            Settings::new(),
            period:              ArgParseTimePeriod::new(),
            unique_option_names: HashSet::new(),
            unique_options:      Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg: String) {
        if self.name.is_none() {
            self.name = Some(arg);
        } else {
            self.args.push(arg);
        }
    }

    pub fn add_setting<F>(&mut self, arg: String, callback: F) -> Result<(), ArgParseError>
    where
        F: FnOnce(&mut Settings) -> Result<(), ArgParseError>,
    {
        self.settings_options.push(arg);
        callback(&mut self.settings)?;
        Ok(())
    }

    pub fn add_time_period_option<F>(&mut self, arg: String, callback: F) -> Result<(), ArgParseError>
    where
        F: FnOnce(&mut ArgParseTimePeriod) -> Result<(), ArgParseError>,
    {
        self.period_options.push(arg);
        callback(&mut self.period)?;
        Ok(())
    }

    pub fn add_unique_option(&mut self, arg: String, value: UniqueOption) -> Result<(), ArgParseError> {
        let is_new = self.unique_option_names.insert(arg.clone());

        if !is_new {
            return Err(arg_parse_error_localized!(
                "More then one '{arg}' option specfied",
                "Указано более одной опции '{arg}'"
            ));
        }

        self.unique_options.push(value);

        Ok(())
    }

    pub fn as_action(self, cmd: &str) -> Result<Action, ArgParseError> {
        if self.name.is_none() {
            return Err(arg_parse_error_localized_with_usage!(
                "No action specified",
                "Действие не указано",
                cmd
            ));
        }

        let name = self.name.as_ref().unwrap().as_str();

        let mut action = match name {
            "help" => return Ok(Action::Help), // Ignore any other options

            "get"    => Action::GetCurrentWallpaper,
            "set"    => Action::SetWallpaper { path: String::new(), settings: Settings::new(), period: NotSpecified },
            "random" => Action::SetRandowWallpaper,

            "list"   => Action::GetNodeList,
            "add"    => Action::AddNodes    { paths: Vec::new(), settings: Settings::new() },
            "remove" => Action::RemoveNodes { paths: Vec::new() },
            "clear"  => Action::ClearNodes,

            "group-list"        => Action::GetGroupList,
            "new-group"         => Action::NewGroup        { name: String::new(), paths: Vec::new() },
            "get-group"         => Action::GetGroup        { name: String::new() },
            "set-group"         => Action::SetGroup        { name: String::new(), settings: Settings::new(), period: NotSpecified },
            "add-to-group"      => Action::AddToGroup      { name: String::new(), paths: Vec::new() },
            "remove-from-group" => Action::RemoveFromGroup { name: String::new(), paths: Vec::new() },
            "clear-group"       => Action::ClearGroup      { name: String::new() },
            "remove-group"      => Action::RemoveGroup     { name: String::new() },

            "find-non-fitting" => Action::FindNonFittingWallpapers { paths: Vec::new(), display_id: None, is_short: false },

            _ => {
                return Err(arg_parse_error_localized_with_usage!(
                    "Unknown action: '{name}'",
                    "Неизвестное действие: '{name}'",
                    cmd
                ));
            }
        };

        action = add_args(name, action, self.args)?;
        action = add_settings(name, action, self.settings, self.settings_options)?;
        action = add_period(name, action, self.period, self.period_options)?;
        action = add_unique_options(name, action, self.unique_options)?;
        action = validate_name(action)?;
        action = validate_paths(action)?;
        Ok(action)
    }
}

fn add_args(action_name: &str, mut action: Action, mut args: Vec<String>) -> Result<Action, ArgParseError> {
    match action {
        Action::AddNodes { ref mut paths, .. } | Action::RemoveNodes { ref mut paths } => {
            match args.len() {
                0 => Err(ArgParseError::at_least_one_path_required(action_name)),
                1.. => {
                    *paths = args;
                    Ok(action)
                }
            }
        }

        Action::SetWallpaper { ref mut path, .. } => {
            *path = require_exactly_one(action_name, args, ArgParseError::path_required)?;
            Ok(action)
        }

        Action::GetGroup    { ref mut name } |
        Action::SetGroup    { ref mut name, .. } |
        Action::ClearGroup  { ref mut name } |
        Action::RemoveGroup { ref mut name } => {
            *name = require_exactly_one(action_name, args, ArgParseError::name_required)?;
            Ok(action)
        }

        Action::NewGroup { ref mut name, ref mut paths } => {
            match args.len() {
                0 => Err(ArgParseError::name_required(action_name)),
                1.. => {
                    *name = args.remove(0);
                    *paths = args;
                    Ok(action)
                }
            }
        }

        Action::AddToGroup      { ref mut name, ref mut paths } |
        Action::RemoveFromGroup { ref mut name, ref mut paths } => {
            match args.len() {
                0 => Err(ArgParseError::name_required(action_name)),
                1 => Err(ArgParseError::at_least_one_path_required(action_name)),
                2.. => {
                    *name = args.remove(0);
                    *paths = args;
                    Ok(action)
                }
            }
        }

        Action::FindNonFittingWallpapers { ref mut paths, .. } => {
            *paths = args;
            Ok(action)
        }

        _ => {
            if args.is_empty() {
                Ok(action)
            } else {
                Err(ArgParseError::unknown_argument(&args[0], action_name))
            }
        }
    }
}

fn require_exactly_one<F>(action_name: &str, mut args: Vec<String>, no_arg_error: F) -> Result<String, ArgParseError>
where
    F: FnOnce(&str) -> ArgParseError,
{
    match args.len() {
        0   => Err(no_arg_error(action_name)),
        2.. => Err(ArgParseError::unknown_argument(&args[1], action_name)),
        1   => Ok(args.remove(0)),
    }
}

fn add_settings(action_name: &str, mut action: Action, parsed_settings: Settings, settings_options: Vec<String>) -> Result<Action, ArgParseError> {
    match action {
        Action::SetWallpaper { ref mut settings, .. } |
        Action::AddNodes     { ref mut settings, .. } |
        Action::SetGroup     { ref mut settings, .. } => {
            assert!(settings.is_none());
            *settings = parsed_settings;
            Ok(action)
        }

        _ => {
            assert_eq!(parsed_settings.is_none(), settings_options.is_empty());

            if parsed_settings.is_none() {
                Ok(action)
            } else {
                Err(ArgParseError::could_not_set_option(
                    settings_options,
                    action_name,
                ))
            }
        }
    }
}

fn add_period(action_name: &str, mut action: Action, parsed_period: ArgParseTimePeriod, period_options: Vec<String>) -> Result<Action, ArgParseError> {
    assert_eq!(parsed_period.is_none(), period_options.is_empty());

    match action {
        Action::SetWallpaper { ref mut period, .. } |
        Action::SetGroup     { ref mut period, .. } => {
            assert!(*period == NotSpecified);
            *period = parsed_period.as_parsed_time_period()?;
            Ok(action)
        }

        _ => {
            if parsed_period.is_none() {
                Ok(action)
            } else {
                Err(ArgParseError::could_not_set_option(
                    period_options,
                    action_name,
                ))
            }
        }
    }
}

fn add_unique_options(action_name: &str, mut action: Action, uniqie_options: Vec<UniqueOption>) -> Result<Action, ArgParseError> {

    for option in uniqie_options {
        match option {
            UniqueOption::DisplayId(id) => match action {
                Action::FindNonFittingWallpapers {
                    ref mut display_id, ..
                } => {
                    *display_id = Some(id);
                }

                _ => {
                    return Err(ArgParseError::could_not_set_option(
                        vec!["--display-id".to_string()],
                        action_name,
                    ));
                }
            },

            UniqueOption::ShortFlag => match action {
                Action::FindNonFittingWallpapers {
                    ref mut is_short, ..
                } => {
                    *is_short = true;
                }

                _ => {
                    return Err(ArgParseError::could_not_set_option(
                        vec!["--display-id".to_string()],
                        action_name,
                    ));
                }
            },
        }
    }

    Ok(action)
}

fn validate_name(action: Action) -> Result<Action, ArgParseError> {
    match action {
        Action::NewGroup        { ref name, .. } |
        Action::GetGroup        { ref name, .. } |
        Action::SetGroup        { ref name, .. } |
        Action::AddToGroup      { ref name, .. } |
        Action::RemoveFromGroup { ref name, .. } |
        Action::ClearGroup      { ref name, .. } |
        Action::RemoveGroup     { ref name, .. } => {
            if name.is_empty() {
                return Err(arg_parse_error_localized!(
                    "Group name can't be empty",
                    "Имя группы не должно быть пустым"
                ));
            }

            if name.chars().all(char::is_whitespace) {
                return Err(arg_parse_error_localized!(
                    "Group name can't contain only whitespaces",
                    "Имя группы не должно содержать только пробелы"
                ));
            }
        }

        _ => {}
    }

    Ok(action)
}

fn validate_paths(mut action: Action) -> Result<Action, ArgParseError> {
    match action {
        Action::SetWallpaper { ref mut path, .. } => {
            *path = util::canonicalize_path(path)?;
        }

        Action::AddNodes                 { ref mut paths, .. } |
        Action::RemoveNodes              { ref mut paths, .. } |
        Action::NewGroup                 { ref mut paths, .. } |
        Action::AddToGroup               { ref mut paths, .. } |
        Action::RemoveFromGroup          { ref mut paths, .. } |
        Action::FindNonFittingWallpapers { ref mut paths, .. } => {
            let mut unique_paths = HashSet::with_capacity(paths.len());
            let mut new_paths = Vec::with_capacity(paths.len());

            for path in paths.iter() {
                let path = util::canonicalize_path(path)?;

                if unique_paths.insert(path.clone()) {
                    new_paths.push(path);
                }
            }

            *paths = new_paths;
        }

        _ => {}
    }

    Ok(action)
}
