use std::env;
use std::path::Path;

use path_absolutize::Absolutize;

const SOCKET_NAME: &str = "ywpmd.sock";
const CONFIG_NAME: &str = "config.yaml";


#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use std::path::PathBuf;

    pub fn get_config_paths() -> Vec<PathBuf> {
        vec![PathBuf::from(const_str::concat!("./debug-files/", CONFIG_NAME))]
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

    pub fn get_config_paths() -> Vec<PathBuf> {
		vec![
			PathBuf::from(const_str::concat!("/etc/ywpm/", CONFIG_NAME)),
			get_path("XDG_CONFIG_HOME", ".config/", const_str::concat!("ywpm/", CONFIG_NAME))
		]
    }

    pub fn get_socket_path() -> String {
		get_path("XDG_RUNTIME_DIR", ".local/share/ywpm/", SOCKET_NAME)
			.to_string_lossy().into_owned()
    }

	fn get_path(env_var: &str, default_path: &str, rest_path: &str) -> PathBuf {
		let var = env::var(env_var)
			.ok().filter(|var| !var.is_empty());

		let mut path = if let Some(runtime_dir) = var {
            PathBuf::from(runtime_dir)

        } else {
            let mut path = env::home_dir().expect("Couldn't get home directory");
            path.push(default_path);
            path
        };

		path.push(rest_path);
		path
	}
}


#[cfg(debug_assertions)]
pub use debug::{get_config_paths, get_socket_path};

#[cfg(not(debug_assertions))]
pub use release::{get_config_paths, get_socket_path};


pub(crate) fn canonicalize_path(cwd: &str, path: &str) -> String {
    Path::new(path)
        .absolutize_from(cwd)
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn get_home_with_slash() -> Option<String> {
    env::home_dir()
        .and_then(|home| home.to_str().map(String::from))
        .filter(|home| !home.is_empty())
        .map(|home| if home.ends_with('/') { home } else { home + "/" })
}

pub(crate) fn replace_home_with_tilde(home_with_slash: &Option<String>, path: &str) -> String {
    if let Some(home) = home_with_slash && path.starts_with(home) {
        String::from("~/") + &path[home.len()..]
    } else {
        String::from(path)
    }
}