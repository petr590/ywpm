use once_cell::sync::Lazy;
use std::{env, fs};

use crate::daemon::arg_parsing::ArgParseError;

pub static IS_RU: Lazy<bool> = Lazy::new(|| {
    env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("ru")
});

const PROGRAM_DIR: &str = "ywpm";
const SOCKET_NAME: &str = "ywpm.sock";
const CONFIG_NAME: &str = "config.yaml";

#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use std::path::PathBuf;

    pub fn get_socket_path() -> PathBuf {
        PathBuf::from(format!("/tmp/{SOCKET_NAME}"))
    }

    pub fn get_config_path() -> PathBuf {
        PathBuf::from(format!("/tmp/{PROGRAM_DIR}/{CONFIG_NAME}"))
    }
}

#[cfg(not(debug_assertions))]
mod release {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    pub fn get_socket_path() -> PathBuf {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            let mut path = PathBuf::from(runtime_dir);
            path.push(SOCKET_NAME);
            return path;
        }

        let mut path = env::home_dir().expect("Couldn't get home directory");
        path.push(format!(".local/share/{SOCKET_NAME}"));
        path
    }

    pub fn get_config_path() -> PathBuf {
        let mut path = env::home_dir().expect("Couldn't get home directory");
        path.push(format!(".config/{PROGRAM_DIR}/{CONFIG_NAME}"));
        path
    }
}

#[cfg(debug_assertions)]
pub use debug::*;

#[cfg(not(debug_assertions))]
pub use release::*;

pub fn canonicalize_path(path: &str) -> Result<String, ArgParseError> {
    fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|err| ArgParseError::new(format!("'{path}': {err}")))
}

#[macro_export]
macro_rules! format_localized {
    ($en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        if *crate::util::IS_RU {
            format!($ru_fmt $(, $arg)*)
        } else {
            format!($en_fmt $(, $arg)*)
        }
    };
}
