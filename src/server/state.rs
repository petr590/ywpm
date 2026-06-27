use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::fs::{self, File, Metadata};

use phf::{Set, phf_set};
use rand::random_range;
use walkdir::WalkDir;

use crate::server::display_mode::DisplayMode;
use crate::server::file_or_dir::FileOrDir;
use crate::server::wallpaper_group::WallpaperGroup;
use crate::server::dtos::StateDto;
use crate::server::action_perform_error::ActionPerformError;



#[derive(Debug, Clone)]
pub struct State {
    current_wallpaper_path: Option<Rc<str>>,
    wallpapers: HashMap<Rc<str>, Rc<FileOrDir>>,
    groups:     HashMap<String, WallpaperGroup>,
}


impl State {
    pub fn new() -> Self {
        Self {
            current_wallpaper_path: Option::None,
            wallpapers: HashMap::new(),
            groups:     HashMap::new(),
        }
    }

    pub fn from(_dto: &StateDto) -> Self {
        todo!();
    }

    pub fn as_dto(&self) -> StateDto {
        StateDto {
            current_wallpaper_path: self.clone_current_wallpaper_path().unwrap_or_default(),

            wallpapers: self.wallpapers.iter()
                .map(|(_path, file_or_dir)| file_or_dir.clone())
                .collect(),
            
            groups: self.groups.iter()
                .map(|(name, group)| (name.clone(), group.as_dto()))
                .collect(),
        }
    }


    pub fn add_wallpaper(&mut self, path: impl Into<Rc<str>>, mode: DisplayMode, recursive_level: u16) -> &mut Rc<FileOrDir> {
        let key = path.into();

        self.wallpapers.entry(key.clone())
            .or_insert(Rc::new(FileOrDir::new(key, mode, recursive_level)))
    }

    pub fn remove_wallpaper<'a>(&mut self, path: impl Into<&'a Rc<str>>) {
        self.wallpapers.remove(path.into());
    }

    pub fn get_wallpaper_mut<'a>(&mut self, path: impl Into<&'a Rc<str>>) -> Option<&mut Rc<FileOrDir>> {
        self.wallpapers.get_mut(path.into())
    }


    pub fn add_group(&mut self, name: impl Into<String>) -> &mut WallpaperGroup {
        self.groups.entry(name.into()).or_insert(WallpaperGroup::new())
    }

    pub fn remove_group<'a>(&mut self, name: impl Into<&'a String>) {
        self.groups.remove(name.into());
    }


    pub fn read_or_create_empty(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        if let Some(dir) = path.as_ref().parent() {
            fs::create_dir_all(dir)?;
        }

        if fs::exists(&path)? {
            let content = fs::read_to_string(&path)?;

            match yaml_serde::from_str(&content) {
                Ok(dto) => Ok(State::from(&dto)),
                Err(_) => State::create_and_write_empty_config(path)
            }

        } else {
            State::create_and_write_empty_config(path)
        }
    }

    fn create_and_write_empty_config(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let state = Self::new();
        let content = yaml_serde::to_string(&state.as_dto())?;
        fs::write(&path, content)?;
        Ok(state)
    }

    pub fn clone_current_wallpaper_path(&self) -> Option<String> {
        self.current_wallpaper_path.as_ref().map(|path| String::from(&**path))
    }

    pub fn set_random_wallpaper(&mut self) -> Result<String, ActionPerformError> {
        let paths = self.all_wallpaper_paths();

        if paths.is_empty() {
            return Err(ActionPerformError::new("Can't set random path because there are no valid paths"));
        }

        let path = &paths[random_range(0..paths.len())];

        self.current_wallpaper_path = Option::Some(path.clone());
        Ok(format!("Current wallpaper set: '{}'", path))
    }

    pub fn all_wallpaper_paths(&self) -> Vec<Rc<str>> {
        let mut paths = Vec::new();
        let mut warning = String::new();

        for file_or_dir in self.wallpapers.values() {
            let path = Path::new(&**file_or_dir.path());

            match fs::metadata(path) {
                Ok(metadata) => process_file_or_dir(&mut paths, &mut warning, file_or_dir, metadata),
                Err(err) => append_to_warning(&mut warning, "couldn't get file metadata", err, file_or_dir.path()),
            }
        }

        paths
    }

    pub fn configure_wallpaper(&mut self, path: impl Into<Rc<str>>, mode: Option<DisplayMode>) -> Result<String, ActionPerformError> {
        let path = path.into();
        let file_or_dir = self.wallpapers.entry(path.clone()).or_insert(Rc::new(FileOrDir::new(path, DisplayMode::new(), 1)));

        if let Some(mode) = mode {
            file_or_dir.mode = mode;
        }

        Ok(String::new())
    }
}


fn process_file_or_dir(paths: &mut Vec<Rc<str>>, warning: &mut String, file_or_dir: &FileOrDir, metadata: Metadata) {
    let path = Path::new(&**file_or_dir.path());

    if metadata.is_file() && ext_matches(path) && check_accessible(path, warning) {
        paths.push(file_or_dir.path().clone());
        return;
    }
    
    if metadata.is_dir() {
        let walk_dir = WalkDir::new(path)
                .follow_links(true)
                .same_file_system(false)
                .max_depth(file_or_dir.recursive_level as usize);

        for entry in walk_dir {
            match entry {
                Ok(dir_entry) => {
                    if dir_entry.file_type().is_file() && ext_matches(dir_entry.path()) && check_accessible(dir_entry.path(), warning) {
                        paths.push(Rc::from(dir_entry.path().to_str().unwrap_or_default()));
                    }
                }

                Err(err) => {
                    if err.io_error().is_some() {
                        append_to_warning(warning, "I/O error", err, file_or_dir.path());

                    } else {
                        warning.push_str("Warning: ");
                        warning.push_str(&err.to_string());
                        warning.push_str("'\n");
                    }
                }
            }
        }

        return;
    }
}



static EXTENSIONS: Set<&str> = phf_set! {
    "jpg", "jpeg", "png", "apng", "webp", "gif", "bmp", "tiff", "tif", "ico", "heic", "heif", "avif",
    "mp4", "m4v", "mkv", "avi", "mov", "webm", "flv", "wmv",
};

fn ext_matches(path: impl AsRef<Path>) -> bool {
    let ext = path.as_ref()
        .extension().unwrap_or_default()
        .to_str().unwrap_or_default();

    return EXTENSIONS.contains(ext);
}

fn check_accessible(path: impl AsRef<Path>, warning: &mut String) -> bool {
    let path_ref = path.as_ref();

    match File::open(path_ref) {
        Ok(_) => true,
        Err(err) => {
            append_to_warning(warning, "couldn't open file", err, path_ref.to_str().unwrap_or_default());
            false
        },
    }
}

fn append_to_warning(warning: &mut String, message: impl AsRef<str>, err: impl Error, path: impl AsRef<str>) {
    warning.push_str("Warning: ");
    warning.push_str(message.as_ref());
    warning.push_str(": ");
    warning.push_str(&err.to_string());
    warning.push_str(": '");
    warning.push_str(path.as_ref());
    warning.push_str("'\n");
}