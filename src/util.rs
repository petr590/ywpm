const PROGRAM_DIR: &str = "ywpm";
const SOCKET_NAME: &str = "socket.sock";
const CONFIG_NAME: &str = "config.yaml";


#[cfg(debug_assertions)]
mod debug {
    use std::path::PathBuf;
    use super::*;

    pub fn get_socket_path() -> PathBuf {
        PathBuf::from(format!("/tmp/{PROGRAM_DIR}/{SOCKET_NAME}"))
    }

    pub fn get_config_path() -> PathBuf {
        PathBuf::from(format!("/tmp/{PROGRAM_DIR}/{CONFIG_NAME}"))
    }
}

#[cfg(not(debug_assertions))]
mod release {
    use std::path::PathBuf;
    use std::env;
    use super::*;

    pub fn get_socket_path() -> PathBuf {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            let mut path = PathBuf::from(runtime_dir);
            path.push(SOCKET_NAME);
            return path;
        }
        
        let mut path = env::home_dir().expect("Couldn't get home directory");
        path.push(format!(".local/share/{PROGRAM_DIR}/{SOCKET_NAME}"));
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