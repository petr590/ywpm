use std::fmt;

use crate::arg_parse_error_localized;

#[derive(Debug, PartialEq)]
pub struct ArgParseError {
    message: String,
}

impl std::error::Error for ArgParseError {}

impl fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ArgParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn unknown_argument(arg: &str, action_name: &str) -> Self {
        arg_parse_error_localized!(
            "Unknown argument '{arg}' for action '{action_name}'",
            "Неизвестный аргумент '{arg}' для действия '{action_name}'",
        )
    }

    pub fn name_required(action_name: &str) -> Self {
        arg_parse_error_localized!(
            "Missing name for action '{action_name}'",
            "Требуется имя для действия '{action_name}'",
        )
    }

    pub fn path_required(action_name: &str) -> Self {
        arg_parse_error_localized!(
            "Missing path for action '{action_name}'",
            "Требуется путь для действия '{action_name}'",
        )
    }

    pub fn at_least_one_path_required(action_name: &str) -> Self {
        arg_parse_error_localized!(
            "At least one path is required for action '{action_name}'",
            "Требуется хотя бы один путь для действия '{action_name}'",
        )
    }

    pub fn could_not_set_option(settings_options: Vec<String>, action_name: &str) -> Self {
        arg_parse_error_localized!(
            "Could not set {} for action '{}'",
            "Невозможно задать {} для действия '{}'",
            format_options(settings_options),
            action_name
        )
    }
}

fn format_options(mut options: Vec<String>) -> String {
    match options.len() {
        0 => panic!("Options list is empty"),
        1 => format!("'{}' option", options[0]),
        _ => {
            let last = options.remove(options.len() - 1);
            format!("'{}', or '{}' options", options.join("', '"), last)
        }
    }
}

#[macro_export]
macro_rules! arg_parse_error_localized {
    ($en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::daemon::arg_parsing::ArgParseError::new(crate::format_localized!($en_fmt, $ru_fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! arg_parse_error_localized_with_usage {
    ($en_fmt:expr, $ru_fmt:expr, $cmd:expr $(, $arg:expr)* $(,)?) => {
        crate::daemon::arg_parsing::ArgParseError::new(
            if *crate::util::IS_RU {
                $crate::indoc::formatdoc! {
                    "
                        {0}
                        Использование: {1} <ДЕЙСТВИЕ> [ОПЦИИ] [АРГУМЕНТЫ]
                        Используйте '{1} --help' для дополнительной информации
                    ",
                    format!($ru_fmt $(, $arg)*),
                    $cmd
                }
            } else {
                $crate::indoc::formatdoc! {
                    "
                        {0}
                        Usage: {1} <ACTION> [OPTIONS] [ARGUMENTS]
                        Run '{1} --help' for more information
                    ",
                    format!($en_fmt $(, $arg)*),
                    $cmd
                }
            }
        )
    };
}
