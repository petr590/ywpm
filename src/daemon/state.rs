use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File, Metadata};
use std::path::Path;

use chrono::{Local, NaiveDateTime};
use itertools::Itertools;
use nix::poll::PollTimeout;
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
use crate::daemon::warning::Warning;



#[derive(Debug, Clone)]
pub struct State {
    current_wallpaper_path: Option<String>,
    nodes:  HashMap<String, SharedWallpaperNode>,
    groups: HashMap<String, WallpaperGroup>,
}


impl State {

    // ----------------------------------------- Creation -----------------------------------------

    pub fn new() -> Self {
        Self {
            current_wallpaper_path: None,
            nodes:  HashMap::new(),
            groups: HashMap::new(),
        }
    }

    pub fn from_dto(dto: StateDto) -> Self {
        let nodes = dto.nodes.into_iter()
                .map(|node| {
                    let path = node.borrow().path().clone();
                    (path, node)
                }).collect();

        let groups = dto.groups.into_iter()
                .map(|(name, group_dto)| (name, WallpaperGroup::from_dto(group_dto, &nodes)))
                .collect();

        Self {
            current_wallpaper_path: dto.current_wallpaper_path.clone(),
            nodes,
            groups,
        }
    }

    pub fn as_dto(&self) -> StateDto {
        StateDto {
            current_wallpaper_path: self.current_wallpaper_path.clone(),

            nodes: self.nodes.iter()
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

        self.nodes.entry(path.clone())
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
        self.find_all_child_files(self.nodes.values()).0
    }


    // ---------------------------------------- Wallpapers ----------------------------------------

    pub fn restore_wallpaper(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_path) = &self.current_wallpaper_path {

            let node = match self.find_closest_node(current_path) {
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

            let (nodes, warning) = self.find_all_child_files([&node]);
            eprint!("{warning}");

            self.run_backend_with_random_wallpaper(&nodes)?;
        }

        Ok(())
    }


    pub fn get_wallpaper_list(&self) -> String {
        if self.nodes.is_empty() {
            String::from("No wallpapers are found")
        } else {
            format!("Wallpapers:\n{}",
                    self.nodes.values()
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
        let node = self.update_node(&path, settings, period.clone());
        
        if period.as_ref().is_none_or(|period| period.is_datetime_in_bounds(&Local::now().naive_local())) {
            self.set_node([&node])
        } else {
            Ok(format_localized!(
                "The wallpaper is scheduled for the {}: '{}'",
                "Установка обоев запланирована на период {}: '{}'",
                period.unwrap(), path
            ))
        }
    }

    pub fn set_random_wallpaper(&mut self) -> Result<String, ActionPerformError> {
        self.set_node(self.find_actual_nodes().iter())
    }


    fn set_node<'a>(&mut self, nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>) -> Result<String, ActionPerformError> {
        let (nodes, warning) = self.find_all_child_files(nodes);

        let node = self.run_backend_with_random_wallpaper(&nodes)
                .map_err(ActionPerformError::from_boxed)?;

        Ok(format_localized!(
            "{}Wallpaper set: '{}'",
            "{}Установлены обои: '{}'",
            warning, node.path()
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
                ActionPerformError::from_boxed(err)
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

            self.nodes.remove(path);
        }
    }

    fn get_or_insert_node(&mut self, path: &str) -> SharedWallpaperNode {
        self.nodes
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
        let group_nodes = paths.iter()
            .map(|path| self.get_or_insert_node(path))
            .collect::<HashSet<SharedWallpaperNode>>();

        self.groups.entry(name.into())
            .or_insert_with(WallpaperGroup::new)
            .add_all(group_nodes);


        Ok(())
    }

    pub fn set_group(&mut self, name: &str, _settings: &Settings, period: Option<TimePeriod>) -> Result<String, ActionPerformError> {
        let group = self.get_group_mut(name)?;
        group.period = period;

        todo!()
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


    // -------------------------------------- Util functions --------------------------------------


    fn find_actual_nodes(&self) -> Vec<SharedWallpaperNode> {
        let now = Local::now().naive_local();

        let nodes = self.nodes.values()
            .filter(|node| node.borrow().period.as_ref().is_some_and(|period| period.is_datetime_in_bounds(&now)));

        let group_nodes = self.groups.values()
            .filter(|group| group.period.as_ref().is_some_and(|period| period.is_datetime_in_bounds(&now)))
            .flat_map(|group| group.nodes());

        let vec = nodes.chain(group_nodes)
            .cloned().unique()
            .collect::<Vec<SharedWallpaperNode>>();

        if vec.is_empty() {
            self.nodes.values().cloned().collect()
        } else {
            vec
        }
    }


    fn find_all_child_files<'a>(&self, nodes: impl IntoIterator<Item = &'a SharedWallpaperNode>) -> (Vec<Wallpaper>, Warning) {
        let mut paths = Vec::new();
        let mut warning = Warning::new();

        for rc in nodes {
            let node = rc.borrow();
            let path = Path::new(node.path());

            match fs::metadata(path) {
                Ok(metadata) => self.add_all_child_files(&mut paths, &mut warning, node, metadata),
                Err(err) => warning.append_msg_error_path("couldn't get file metadata", &err, node.path()),
            }
        }

        (paths, warning)
    }



    fn add_all_child_files(&self, paths: &mut Vec<Wallpaper>, warning: &mut Warning, current_node: Ref<WallpaperNode>, metadata: Metadata) {
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
                            let path = dir_entry.path().to_str().unwrap_or_default();

                            paths.push(Wallpaper::new(
                                String::from(path),
                                self.find_closest_node(path)
                                    .map(|node| node.borrow().mode.clone())
                                    .unwrap_or_else(|| current_node.mode.clone())
                            ));
                        }
                    }

                    Err(err) => {
                        if err.io_error().is_some() {
                            warning.append_msg_error_path("I/O error", &err, current_node.path());

                        } else {
                            warning.append_msg_error("Walkdir error", &err);
                        }
                    }
                }
            }

            return;
        }
    }

    fn find_closest_node(&self, path: &str) -> Option<SharedWallpaperNode> {
        self.nodes.values()
            .filter(|node| path.starts_with(node.borrow().path()))
            .max_by_key(|node| node.borrow().path().len())
            .cloned()
    }


    // ------------------------------------------ Other -------------------------------------------

    pub fn get_timeout(&self) -> PollTimeout {
        let now = Local::now().naive_local();

        let time = self.nodes.values()
            .filter_map(
                |node| node.borrow().period.as_ref()
                    .map(|peroid| peroid.since.clone())
                    .filter(|time| *time > now)
            ).min();

        if let Some(time) = time {
            let duration: Result<i32, _> = (time - now).num_milliseconds().try_into();

            match duration {
                Ok(num) => PollTimeout::try_from(num).unwrap(),
                Err(_) => PollTimeout::MAX
            }

        } else {
            PollTimeout::NONE
        }
    }

    pub fn clear_expired_peroids(&mut self) {
        let now = Local::now().naive_local();

        for node in self.nodes.values_mut() {
            clear_period_if_expired(&mut node.borrow_mut().period, &now);
        }

        for group in self.groups.values_mut() {
            clear_period_if_expired(&mut group.period, &now);
        }
    }
}


// ---------------------------------------- Util functions ----------------------------------------


fn clear_period_if_expired(period: &mut Option<TimePeriod>, now: &NaiveDateTime) {
    if period.as_ref().is_some_and(|period| period.is_expired(now)) {
        *period = None;
    }
}


fn group_not_found_error(name: &str) -> ActionPerformError {
    action_perform_error_localized!(
        "Group '{name}' not found",
        "Группа '{name}' не найдена"
    )
}



static EXTENSIONS: Set<&str> = phf_set! {
    "jpg", "jpeg", "png", "apng", "webp", "gif", "bmp", "tiff", "tif", "ico", "heic", "heif", "avif",
    "mp4", "m4v", "mkv", "avi", "mov", "webm", "flv", "wmv",
};

fn ext_matches(path: impl AsRef<Path>) -> bool {
    path.as_ref().extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| EXTENSIONS.contains(ext))
}

fn check_accessible(path: impl AsRef<Path>, warning: &mut Warning) -> bool {
    let path_ref = path.as_ref();

    match File::open(path_ref) {
        Ok(_) => true,
        Err(err) => {
            warning.append_msg_error_path("couldn't open file", &err, path_ref.to_str().unwrap_or_default());
            false
        },
    }
}