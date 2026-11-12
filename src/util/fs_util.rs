use std::borrow::Cow;
use std::env;
use std::path::Path;

use nix::NixPath;

const SOCKET_NAME: &str = "ywpmd.sock";
const CONFIG_NAME: &str = "config.yaml";


#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use std::path::PathBuf;

    pub fn get_config_paths() -> Vec<PathBuf> {
        vec![PathBuf::from(const_str::concat!("./debug-files/", CONFIG_NAME))]
    }

    pub fn get_socket_path() -> PathBuf {
        PathBuf::from(const_str::concat!("/tmp/", SOCKET_NAME))
    }
}

#[cfg(not(debug_assertions))]
mod release {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    pub fn get_config_paths() -> Vec<PathBuf> {
		vec![
			get_path("XDG_CONFIG_HOME", ".config/", const_str::concat!("ywpm/", CONFIG_NAME)),
			PathBuf::from(const_str::concat!("/etc/ywpm/", CONFIG_NAME)),
		]
    }

    pub fn get_socket_path() -> PathBuf {
		get_path("XDG_RUNTIME_DIR", ".local/share/ywpm/", SOCKET_NAME)
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


pub(crate) fn replace_tilde_with_home(path: &Path) -> Cow<'_, Path> {
    if  let Some(home) = env::home_dir() &&
        let Ok(remainder) = path.strip_prefix("~")
    {    
        Cow::Owned(home.join(remainder))
    } else {
        Cow::Borrowed(path)
    }
}

pub(crate) fn replace_home_with_tilde(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();

    if  let Some(home) = env::home_dir() && 
        let Ok(remainder) = path.strip_prefix(home)
    {
        let mut result = String::with_capacity(remainder.len() + 2);
        result.push_str("~/");
        result.push_str(&remainder.to_string_lossy());
        result

    } else {
        path.to_string_lossy().into_owned()
    }
}