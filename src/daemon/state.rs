use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File, Metadata};
use std::path::Path;

use itertools::Itertools;
use phf::{Set, phf_set};
use rand::random_range;
use walkdir::WalkDir;

use crate::{action_perform_error_localized, format_localized};
use crate::daemon::action_perform_error::ActionPerformError;
use crate::daemon::backend;
use crate::daemon::display_mode::DisplayMode;
use crate::daemon::dtos::StateDto;
use crate::daemon::settings::Settings;
use crate::daemon::time_period::TimePeriod;
use crate::daemon::wallpaper::Wallpaper;
use crate::daemon::wallpaper_group::WallpaperGroup;
use crate::daemon::wallpaper_node::{WallpaperNode, SharedWallpaperNode};



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
            current_wallpaper_path: None,
            wallpapers: HashMap::new(),
            groups:     HashMap::new(),
        }
    }

    pub fn from_dto(dto: StateDto) -> Self {
        let wallpapers = dto.wallpapers.into_iter()
                .map(|node| {
                    let path = node.borrow().path().clone();
                    (path, node)
                }).collect();

        let groups = dto.groups.into_iter()
                .map(|(name, group_dto)| (name, WallpaperGroup::from_dto(&group_dto, &wallpapers)))
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
                Ok(dto) => Ok(State::from_dto(dto)),
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


    // ------------------------------------------ Tests -------------------------------------------

    #[cfg(test)]
    pub(crate) fn add_wallpaper(&mut self, path: impl Into<String>, mode: DisplayMode, recursive_level: u16, period: Option<TimePeriod>) -> &mut SharedWallpaperNode {
        let path = path.into();

        self.wallpapers.entry(path.clone())
            .or_insert(SharedWallpaperNode::new(path, mode, recursive_level, period))
    }

    #[cfg(test)]
    pub(crate) fn add_group(&mut self, name: impl Into<String>) -> &mut WallpaperGroup {
        self.groups
            .entry(name.into())
            .or_insert_with(|| WallpaperGroup::new())
    }

    #[cfg(test)]
    pub(crate) fn find_all_wallpapers(&self) -> Vec<Wallpaper> {
        find_all_child_files(self.wallpapers.values()).0
    }


    // ----------------------------------------- Backend ------------------------------------------

    pub fn start_backend(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_path) = &self.current_wallpaper_path {

            let opt_node = self.wallpapers.iter()
                    .filter_map(|(path, node)| if current_path.starts_with(path) { Some(node) } else { None })
                    .min_by_key(|node| node.borrow().path().len())
                    .cloned();

            let node = match opt_node {
                Some(node) => {
                    let node = node.borrow();
                    SharedWallpaperNode::new(
                        current_path,
                        node.mode.clone(),
                        node.recursive_level,
                        node.period.clone()
                    )
                },

                None => self.get_or_insert_node(&current_path.clone())
            };

            let (nodes, warning) = find_all_child_files([&node]);

            if !warning.is_empty() {
                eprintln!("{warning}");
            }

            self.run_backend_with_random_wallpaper(&nodes)?;
        }

        Ok(())
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

    pub fn set_wallpaper(&mut self, path: impl Into<String>, settings: &Settings, period: Option<TimePeriod>) -> Result<String, ActionPerformError> {
        let path = path.into();
        let node = self.update_node(&path, settings, period);

        let (nodes, warning) = find_all_child_files([&node]);

        let node = self.run_backend_with_random_wallpaper(&nodes)
                .map_err(ActionPerformError::from_unknown_error)?;

        match backend::run(&node) {
            Ok(()) => {
                self.current_wallpaper_path = Some(path);
                Ok(format_localized!(
                    "{}Wallpaper set: '{}'",
                    "{}Установлены обои: '{}'",
                    append_ln_if_not_empty(warning), node.path()
                ))
            },
            Err(err) => {
                self.current_wallpaper_path = None;
                Err(ActionPerformError::from_unknown_error(err))
            }
        }
    }

    pub fn set_random_wallpaper(&mut self) -> Result<String, ActionPerformError> {

        let (nodes, warning) = find_all_child_files(self.wallpapers.values());

        let node = self.run_backend_with_random_wallpaper(&nodes)
                .map_err(ActionPerformError::from_unknown_error)?;

        Ok(format_localized!(
            "{}Wallpaper set: '{}'",
            "{}Установлены обои: '{}'",
            append_ln_if_not_empty(warning), node.path()
        ))
    }


    fn run_backend_with_random_wallpaper<'a>(&mut self, wallpapers: &'a Vec<Wallpaper>) -> Result<&'a Wallpaper, Box<dyn Error>> {

        if wallpapers.is_empty() {

            backend::stop();
            self.current_wallpaper_path = None;

            return Err(Box::new(action_perform_error_localized!(
                "Can't set random wallpaper because there are no valid paths",
                "Не удалось установить случайные обои, так как нет допустимых путей"
            )));
        }

        let wallpaper = &wallpapers[random_range(0..wallpapers.len())];

        match backend::run(wallpaper) {
            Ok(()) => {
                self.current_wallpaper_path = Some(wallpaper.path().clone());
                Ok(wallpaper)
            },

            Err(err) => {
                self.current_wallpaper_path = None;
                Err(err)
            },
        }
    }
    

    // ------------------------------------------ Nodes -------------------------------------------

    pub fn add_nodes(&mut self, paths: &Vec<String>, settings: &Settings) -> Result<(), ActionPerformError> {
        let mut update_current = false;

        for path in paths {
            self.update_node(path, settings, None);
            update_current = update_current || self.current_wallpaper_path.as_ref().is_some_and(|p| path.starts_with(p));
        }

        if update_current {
            let node = self.get_or_insert_node(&self.current_wallpaper_path.clone().unwrap());

            backend::run(&Wallpaper::from(&*node.borrow())).map_err(|err| {
                self.current_wallpaper_path = None;
                ActionPerformError::from_unknown_error(err)
            })?;
        }

        Ok(())
    }

    pub fn remove_nodes(&mut self, paths: &Vec<String>) {
        for path in paths {

            if self.current_wallpaper_path.as_ref().is_some_and(|p| *p == *path) {
                backend::stop();
                self.current_wallpaper_path = None;
            }

            for (_, group) in &mut self.groups {
                group.remove(path);
            }

            self.wallpapers.remove(path);
        }
    }

    fn get_or_insert_node(&mut self, path: &str) -> SharedWallpaperNode {
        self.wallpapers
            .entry(String::from(path))
            .or_insert_with(|| SharedWallpaperNode::new(path, DisplayMode::new(), 1, None))
            .clone()
    }

    fn update_node(&mut self, path: &str, settings: &Settings, period: Option<TimePeriod>) -> SharedWallpaperNode {
        let node = self.get_or_insert_node(path);
        settings.update_node(&mut node.borrow_mut());

        if period.is_some() {
            node.borrow_mut().period = period;
        }

        node
    }


    // ------------------------------------------ Groups ------------------------------------------

    pub fn get_group_list(&self) -> String {
        if self.groups.is_empty() {
            format_localized!(
                "No groups are found",
                "Ни одна группа не найдена"
            )
        } else {
            format_localized!(
                "Groups: ['{}']",
                "Группы: ['{}']",
                self.groups.keys().format("', '")
            )
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

    pub fn set_group(&mut self, _name: &str, _settings: &Settings, _peroid: &Option<TimePeriod>) -> Result<String, ActionPerformError> {
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
    action_perform_error_localized!(
        "Group '{name}' not found",
        "Группа '{name}' не найдена"
    )
}



fn find_all_child_files<'a>(nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>) -> (Vec<Wallpaper>, String) {
    let mut paths = Vec::new();
    let mut warning = String::new();

    for rc in nodes {
        let node = rc.borrow();
        let path = Path::new(node.path());

        match fs::metadata(path) {
            Ok(metadata) => add_all_child_files(&mut paths, &mut warning, node, metadata),
            Err(err) => append_to_warning(&mut warning, "couldn't get file metadata", err, node.path()),
        }
    }

    (paths, warning)
}



fn add_all_child_files(paths: &mut Vec<Wallpaper>, warning: &mut String, current_node: Ref<WallpaperNode>, metadata: Metadata) {
    let path = Path::new(current_node.path());

    if metadata.is_file() && ext_matches(path) && check_accessible(path, warning) {
        paths.push(Wallpaper::from(&*current_node));
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
                        paths.push(Wallpaper::new(
                            String::from(dir_entry.path().to_str().unwrap_or_default()),
                            current_node.mode.clone() // TODO find closest node
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

fn append_ln_if_not_empty(mut s: String) -> String {
    if !s.is_empty() {
        s.push('\n');
    }

    s
}