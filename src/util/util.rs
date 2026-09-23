use std::env;
use std::path::Path;

use once_cell::sync::Lazy;
use path_absolutize::Absolutize;

pub static IS_RU: Lazy<bool> = Lazy::new(|| {
    env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("ru")
});

const PROGRAM_DIR: &str = "ywpm";
const SOCKET_NAME: &str = "ywpmd.sock";
const CONFIG_NAME: &str = "config.yaml";


#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use std::path::PathBuf;

    pub fn get_config_path() -> PathBuf {
        PathBuf::from(const_str::concat!("/tmp/", PROGRAM_DIR, "/", CONFIG_NAME))
    }

    pub fn get_socket_path() -> String {
        String::from(const_str::concat!("/tmp/", SOCKET_NAME))
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

    pub fn get_socket_path() -> String {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            let mut path = PathBuf::from(runtime_dir);
            path.push(SOCKET_NAME);
            path.to_string_lossy().into_owned()

        } else {
            let mut path = env::home_dir().expect("Couldn't get home directory");
            path.push(const_str::concat!(".local/share/", SOCKET_NAME));
            path.to_string_lossy().into_owned()
        }
    }
}

#[cfg(debug_assertions)]
pub use debug::{get_config_path, get_socket_path};

#[cfg(not(debug_assertions))]
pub use release::{get_config_path, get_socket_path};


pub fn canonicalize_path(cwd: &str, path: &str) -> String {
    Path::new(path)
        .absolutize_from(cwd)
        .to_string_lossy()
        .into_owned()
}

#[macro_export]
macro_rules! str_localized {
    ($en_msg:expr, $ru_msg:expr) => {
        if *crate::util::IS_RU {
            $ru_msg
        } else {
            $en_msg
        }
    };
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
