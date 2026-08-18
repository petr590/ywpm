use crate::daemon::action::Action;
use crate::daemon::arg_parsing::error::ArgParseError;
use crate::daemon::arg_parsing::time_period::ArgParseTimePeriod;
use crate::daemon::settings::Settings;
use crate::{arg_parse_error_localized, arg_parse_error_localized_with_usage, util};

#[derive(PartialEq)]
pub(crate) struct ArgParseAction {
    name: Option<String>,
    args: Vec<String>,

    settings_options: Vec<String>,
    period_options:   Vec<String>,

    settings: Settings,
    period:   ArgParseTimePeriod,
}

impl ArgParseAction {
    pub fn new() -> Self {
        Self {
            name:             None,
            args:             Vec::new(),
            settings_options: Vec::new(),
            period_options:   Vec::new(),
            settings:         Settings::new(),
            period:           ArgParseTimePeriod::new(),
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
        F: FnOnce(&mut Settings) -> Result<(), ArgParseError> {
        
        self.settings_options.push(arg);
        callback(&mut self.settings)?;
        Ok(())
    }

    pub fn add_time_period_option<F>(&mut self, arg: String, callback: F) -> Result<(), ArgParseError>
    where 
        F: FnOnce(&mut ArgParseTimePeriod) -> Result<(), ArgParseError> {
        
        self.period_options.push(arg);
        callback(&mut self.period)?;
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

            "list"   => Action::GetWallpaperList,
            "get"    => Action::GetWallpaper,
            "set"    => Action::SetWallpaper { path: String::new(), settings: Settings::new(), period: None },
            "random" => Action::SetRandowWallpaper,
            
            "add"    => Action::AddNodes    { paths: Vec::new(), settings: Settings::new() },
            "remove" => Action::RemoveNodes { paths: Vec::new() },

            "group-list"        => Action::GetGroupList,
            "new-group"         => Action::NewGroup        { name: String::new(), paths: Vec::new() },
            "get-group"         => Action::GetGroup        { name: String::new() },
            "set-group"         => Action::SetGroup        { name: String::new(), settings: Settings::new(), period: None },
            "add-to-group"      => Action::AddToGroup      { name: String::new(), paths: Vec::new() },
            "remove-from-group" => Action::RemoveFromGroup { name: String::new(), paths: Vec::new() },
            "clear-group"       => Action::ClearGroup      { name: String::new() },
            "remove-group"      => Action::RemoveGroup     { name: String::new() },

            _ => return Err(arg_parse_error_localized_with_usage!(
                "Unknown action: '{name}'",
                "Неизвестное действие: '{name}'",
                cmd
            ))
        };

        action = add_args     (name, action, self.args)?;
        action = add_settings (name, action, self.settings, self.settings_options)?;
        action = add_peroid   (name, action, self.period, self.period_options)?;
        action = validate_name  (action)?;
        action = validate_paths (action)?;
        Ok(action)
    }
}

fn add_args(action_name: &str, mut action: Action, mut args: Vec<String>) -> Result<Action, ArgParseError> {
    match action {
        Action::AddNodes    { ref mut paths, .. } |
        Action::RemoveNodes { ref mut paths } => {
            match args.len() {
                0   => Err(ArgParseError::at_least_one_path_required(action_name)),
                1.. => {
                    *paths = args;
                    Ok(action)
                }
            }
        }

        Action::SetWallpaper { ref mut path, .. } => {
            match args.len() {
                0   => Err(ArgParseError::path_required(action_name)),
                2.. => Err(ArgParseError::unknown_argument(&args[1], action_name)),

                1 => {
                    *path = args.remove(0);
                    Ok(action)
                }
            }
        }

        Action::GetGroup    { ref mut name } |
        Action::SetGroup    { ref mut name, .. } |
        Action::ClearGroup  { ref mut name } |
        Action::RemoveGroup { ref mut name } => {
            match args.len() {
                0   => Err(ArgParseError::name_required(action_name)),
                2.. => Err(ArgParseError::unknown_argument(&args[1], action_name)),

                1 => {
                    *name = args.remove(0);
                    Ok(action)
                }
            }
        }

        Action::NewGroup { ref mut name, ref mut paths } => {
            match args.len() {
                0   => Err(ArgParseError::name_required(action_name)),
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
                0   => Err(ArgParseError::name_required(action_name)),
                1   => Err(ArgParseError::at_least_one_path_required(action_name)),
                2.. => {
                    *name = args.remove(0);
                    *paths = args;
                    Ok(action)
                }
            }
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
                Err(ArgParseError::could_not_set_option(settings_options, action_name))
            }
        }
    }
}

fn add_peroid(action_name: &str, mut action: Action, parsed_period: ArgParseTimePeriod, peroid_options: Vec<String>) -> Result<Action, ArgParseError> {
    match action {
        Action::SetWallpaper { ref mut period, .. } |
        Action::SetGroup     { ref mut period, .. } => {
            assert!(period.is_none());
            *period = parsed_period.as_time_period_opt()?;
            Ok(action)
        }

        _ => {
            assert_eq!(parsed_period.is_none(), peroid_options.is_empty());

            if parsed_period.is_none() {
                Ok(action)
            } else {
                Err(ArgParseError::could_not_set_option(peroid_options, action_name))
            }
        }
    }
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
        },

        Action::AddNodes        { ref mut paths, .. } |
        Action::RemoveNodes     { ref mut paths, .. } |
        Action::NewGroup        { ref mut paths, .. } |
        Action::AddToGroup      { ref mut paths, .. } |
        Action::RemoveFromGroup { ref mut paths, .. } => {
            for path in paths {
                *path = util::canonicalize_path(path)?;
            }
        }

        _ => {}
    }

    Ok(action)
}