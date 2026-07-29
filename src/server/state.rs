use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::error::Error;
use std::fs::{self, File, Metadata};
use itertools::Itertools;

use phf::{Set, phf_set};
use rand::random_range;
use walkdir::WalkDir;

use crate::server::backend::{run_backend, stop_backend};
use crate::server::display_mode::DisplayMode;
use crate::server::wallpaper_node::{WallpaperNode, SharedWallpaperNode};
use crate::server::settings::Settings;
use crate::server::wallpaper_group::WallpaperGroup;
use crate::server::dtos::StateDto;
use crate::server::action_perform_error::ActionPerformError;



#[derive(Debug, Clone)]
pub struct State {
    current_wallpaper_path: Option<String>,
    wallpapers: HashMap<String, SharedWallpaperNode>,
    groups:     HashMap<String, WallpaperGroup>,
}


impl State {

    // ----------------------------------------- Creation -----------------------------------------

    pub fn new() -> Self {
        Self {
            current_wallpaper_path: Option::None,
            wallpapers: HashMap::new(),
            groups:     HashMap::new(),
        }
    }

    pub fn from(dto: &StateDto) -> Self {
        let wallpapers = dto.wallpapers.iter()
                .map(|node| (node.borrow().path().clone(), node.clone()))
                .collect();

        let groups = dto.groups.iter()
                .map(|(name, group_dto)| (name.clone(), WallpaperGroup::from(group_dto, &wallpapers)))
                .collect();

        Self {
            current_wallpaper_path: dto.current_wallpaper_path.clone(),
            wallpapers,
            groups,
        }
    }

    pub fn as_dto(&self) -> StateDto {
        StateDto {
            current_wallpaper_path: self.current_wallpaper_path.clone(),

            wallpapers: self.wallpapers.iter()
                .map(|(_path, node)| node.clone())
                .collect(),
            
            groups: self.groups.iter()
                .map(|(name, group)| (name.clone(), group.as_dto()))
                .collect(),
        }
    }


    // ---------------------------------------- Read/write ----------------------------------------

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
        state.write_to(path)?;
        Ok(state)
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let content = yaml_serde::to_string(&self.as_dto())?;
        fs::write(&path, content)?;
        Ok(())
    }


    // ----------------------------------------- Backend ------------------------------------------

    pub fn start_backend(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_path) = &self.current_wallpaper_path {

            let opt_node = self.wallpapers.iter()
                    .filter_map(|(path, node)| if current_path.starts_with(path) { Some(node) } else { None })
                    .max_by_key(|node| node.borrow().path().len())
                    .cloned();

            let node = match opt_node {
                Some(node) => SharedWallpaperNode::new(
                    current_path,
                    node.borrow().mode.clone(),
                    node.borrow().recursive_level.clone()
                ),

                None => self.get_or_insert_node(&current_path.clone())
            };

            run_backend(&node.borrow())?;
        }

        Ok(())
    }


    // ------------------------------------------ Debug -------------------------------------------

    #[cfg(debug_assertions)]
    pub fn add_wallpaper(&mut self, path: impl Into<String>, mode: DisplayMode, recursive_level: u16) -> &mut SharedWallpaperNode {
        let key = path.into();

        self.wallpapers.entry(key.clone())
            .or_insert(SharedWallpaperNode::new(key, mode, recursive_level))
    }

    #[cfg(debug_assertions)]
    pub fn add_group(&mut self, name: impl Into<String>) -> &mut WallpaperGroup {
        self.groups
            .entry(name.into())
            .or_insert_with(|| WallpaperGroup::new())
    }


    // ---------------------------------------- Wallpapers ----------------------------------------

    pub fn get_wallpaper_list(&self) -> String {
        if self.wallpapers.is_empty() {
            String::from("No wallpapers are found")
        } else {
            format!("Wallpapers:\n{}",
                    self.wallpapers.values()
                        .map(|node| node.borrow().path().clone())
                        .format("\n")
            )
        }
    }

    pub fn get_current_wallpaper_path(&self) -> &Option<String> {
        &self.current_wallpaper_path
    }

    fn is_current_wallpaper_path_equals(&self, path: &str) -> bool {
        self.current_wallpaper_path.as_ref().is_some_and(|current_path| *current_path == *path)
    }

    pub fn set_wallpaper(&mut self, path: impl Into<String>, settings: &Settings) -> Result<(), ActionPerformError> {
        let path = path.into();
        let node = self.update_node(&path, settings);

        match run_backend(&node.borrow()) {
            Ok(()) => {
                self.current_wallpaper_path = Some(path);
                Ok(())
            },
            Err(err) => {
                self.current_wallpaper_path = None;
                Err(ActionPerformError::new(err.to_string()))
            }
        }
    }

    pub fn set_random_wallpaper(&mut self) -> Result<String, ActionPerformError> {
        let (nodes, mut warning) = self.all_wallpapers();

        if nodes.is_empty() {
            return Err(ActionPerformError::new("Can't set random path because there are no valid paths"));
        }

        let node = &nodes[random_range(0..nodes.len())];

        self.current_wallpaper_path = Option::Some(node.path().clone());

        match run_backend(node) {
            Ok(()) => {
                if !warning.is_empty() {
                    warning.push('\n');
                }

                Ok(format!("{}Current wallpaper set: '{}'", warning, node.path()))
            },

            Err(err) => Err(ActionPerformError::new(err.to_string())),
        }
    }

    pub fn all_wallpapers(&self) -> (Vec<WallpaperNode>, String) {
        let mut paths = Vec::new();
        let mut warning = String::new();

        for rc in self.wallpapers.values() {
            let node = rc.borrow();
            let path = Path::new(node.path());

            match fs::metadata(path) {
                Ok(metadata) => process_node(&mut paths, &mut warning, node, metadata),
                Err(err) => append_to_warning(&mut warning, "couldn't get file metadata", err, node.path()),
            }
        }

        (paths, warning)
    }
    

    // ------------------------------------------ Nodes -------------------------------------------

    pub fn add_nodes(&mut self, paths: &Vec<String>, settings: &Settings) -> Result<(), ActionPerformError> {
        let mut update_current = false;

        for path in paths {
            self.update_node(path, settings);
            update_current = update_current || self.is_current_wallpaper_path_equals(path);
        }

        if update_current {
            let node = self.get_or_insert_node(&self.current_wallpaper_path.clone().unwrap());

            match run_backend(&node.borrow()) {
                Ok(()) => {},
                Err(err) => {
                    self.current_wallpaper_path = None;
                    return Err(ActionPerformError::new(err.to_string()));
                }
            }
        }

        Ok(())
    }

    pub fn remove_nodes(&mut self, paths: &Vec<String>) {
        for path in paths {
            if self.is_current_wallpaper_path_equals(path) {
                stop_backend();
            }

            for (_, group) in &mut self.groups {
                group.remove(path);
            }

            self.wallpapers.remove(path);
        }
    }

    fn get_or_insert_node(&mut self, path: &str) -> SharedWallpaperNode {
        let path = normalize_path(path);

        self.wallpapers
            .entry(path.clone())
            .or_insert_with(|| SharedWallpaperNode::new(path, DisplayMode::new(), 1))
            .clone()
    }

    fn update_node(&mut self, path: &str, settings: &Settings) -> SharedWallpaperNode {
        let node = self.get_or_insert_node(path);
        settings.update_node(&mut node.borrow_mut());
        node
    }


    // ------------------------------------------ Groups ------------------------------------------

    pub fn get_group_list(&self) -> String {
        if self.groups.is_empty() {
            String::from("No groups are found")
        } else {
            format!("Groups: [{}]", self.groups.keys().format(", "))
        }
    }

    pub fn get_group_info(&self, name: &str) -> Result<String, ActionPerformError> {
        match self.groups.get(name) {
            Some(group) => {
                yaml_serde::to_string(&group.as_dto())
                    .map_err(|err| ActionPerformError::new(err.to_string()))
            }

            None => Err(group_not_found_error(name))
        }
    }

    pub fn new_group(&mut self, name: impl Into<String>, paths: &Vec<String>) -> Result<(), ActionPerformError> {
        let group_wallpapers = paths.iter()
            .map(|path| self.get_or_insert_node(path))
            .collect::<HashSet<SharedWallpaperNode>>();

        self.groups.entry(name.into())
            .or_insert_with(WallpaperGroup::new)
            .add_all(group_wallpapers);


        Ok(())
    }

    pub fn set_group(&mut self, name: &str, settings: &Settings) -> Result<String, ActionPerformError> {
        Ok(String::new())
    }

    pub fn remove_group(&mut self, name: &str) {
        self.groups.remove(name);
    }


    pub fn add_to_group(&mut self, name: impl Into<String>, paths: &Vec<String>) -> Result<(), ActionPerformError> {
        self.new_group(name, paths)
    }

    pub fn remove_from_group(&mut self, name: &str, paths: &Vec<String>) -> Result<(), ActionPerformError> {
        let group = self.get_group_mut(name)?;

        for path in paths {
            group.remove(&path);
        }

        Ok(())
    }

    pub fn clear_group(&mut self, name: &str) -> Result<(), ActionPerformError> {
        self.get_group_mut(name)?.clear();
        Ok(())
    }

    fn get_group_mut(&mut self, name: &str) -> Result<&mut WallpaperGroup, ActionPerformError> {
        self.groups
            .get_mut(name)
            .ok_or_else(|| group_not_found_error(name))
    }
}


// ---------------------------------------- Util functions ----------------------------------------

fn group_not_found_error(name: &str) -> ActionPerformError {
    ActionPerformError::new(format!("Group '{name}' not found"))
}

fn normalize_path(src_path: &str) -> String {
    std::path::absolute(src_path)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| src_path.to_string())
}


fn process_node(paths: &mut Vec<WallpaperNode>, warning: &mut String, current_node: Ref<WallpaperNode>, metadata: Metadata) {
    let path = Path::new(current_node.path());

    if metadata.is_file() && ext_matches(path) && check_accessible(path, warning) {
        paths.push(current_node.clone());
        return;
    }
    
    if metadata.is_dir() {
        let walk_dir = WalkDir::new(path)
                .follow_links(true)
                .same_file_system(false)
                .max_depth(current_node.recursive_level as usize);

        for entry in walk_dir {
            match entry {
                Ok(dir_entry) => {
                    if dir_entry.file_type().is_file() && ext_matches(dir_entry.path()) && check_accessible(dir_entry.path(), warning) {
                        paths.push(WallpaperNode::new(
                            String::from(dir_entry.path().to_str().unwrap_or_default()),
                            current_node.mode.clone(),
                            current_node.recursive_level.clone()
                        ));
                    }
                }

                Err(err) => {
                    if err.io_error().is_some() {
                        append_to_warning(warning, "I/O error", err, current_node.path());

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