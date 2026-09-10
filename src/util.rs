use once_cell::sync::Lazy;
use std::{cell::RefCell, env, fs, path::PathBuf};

use crate::daemon::arg_parsing::ArgParseError;

pub static IS_RU: Lazy<bool> = Lazy::new(|| {
    env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("ru")
});

const PROGRAM_DIR: &str = "ywpm";
const SOCKET_NAME: &str = "ywpmd.sock";
const CONFIG_NAME: &str = "config.yaml";

thread_local! {
    static SOCKET_PATH: RefCell<Option<String>> = RefCell::new(None);
}

pub fn set_socket_path(path: impl Into<String>) {
    SOCKET_PATH.with_borrow_mut(|socket_path| {
        *socket_path = Some(path.into());
    });
}


#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use std::path::PathBuf;

    pub fn get_config_path() -> PathBuf {
        PathBuf::from(const_str::concat!("/tmp/", PROGRAM_DIR, "/", CONFIG_NAME))
    }

    pub(super) fn get_default_socket_path() -> PathBuf {
        PathBuf::from(const_str::concat!("/tmp/", SOCKET_NAME))
    }
}

#[cfg(not(debug_assertions))]
mod release {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    pub fn get_config_path() -> PathBuf {
        let mut path = env::home_dir().expect("Couldn't get home directory");
        path.push(const_str::concat!(".config/", PROGRAM_DIR, "/", CONFIG_NAME));
        path
    }

    pub(super) fn get_default_socket_path() -> PathBuf {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            let mut path = PathBuf::from(runtime_dir);
            path.push(SOCKET_NAME);
            return path;
        }

        let mut path = env::home_dir().expect("Couldn't get home directory");
        path.push(const_str::concat!(".local/share/", SOCKET_NAME));
        path
    }
}

#[cfg(debug_assertions)] pub use debug::get_config_path;
#[cfg(debug_assertions)] use debug::get_default_socket_path;

#[cfg(not(debug_assertions))] pub use release::get_config_path;
#[cfg(not(debug_assertions))] use release::get_default_socket_path;


pub fn get_socket_path() -> PathBuf {
    SOCKET_PATH.with_borrow(|path| {
        if let Some(path) = path {
            PathBuf::from(path)
        } else {
            get_default_socket_path()
        }
    })
}


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
