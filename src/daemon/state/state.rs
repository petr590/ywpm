use std::cell::Ref;
use std::collections::HashMap;
use std::fs::{self, File, Metadata};
use std::path::Path;

use chrono::{Local, NaiveDateTime};
use indexmap::IndexSet;
use itertools::Itertools;
use nix::poll::PollTimeout;
use phf::{Set, phf_set};
use rand::random_range;
use walkdir::WalkDir;

use crate::action_perform_error_localized;
use crate::daemon::action::ActionPerformError;
use crate::daemon::arg_parsing::{ParsedTimePeriod, Settings};
use crate::daemon::backend;
use crate::daemon::state::dtos::StateDto;
use crate::daemon::state::{
    DisplayMode, SharedWallpaperNode, TimePeriod, Wallpaper, WallpaperGroup, WallpaperNode,
};
use crate::daemon::warning::Warning;

#[derive(Debug, Clone)]
pub struct State {
    pub current_wallpaper_path: Option<String>,
    pub nodes: HashMap<String, SharedWallpaperNode>,
    pub groups: HashMap<String, WallpaperGroup>,
}

impl State {
    // ----------------------------------------- Creation -----------------------------------------

    pub fn new() -> Self {
        Self {
            current_wallpaper_path: None,
            nodes: HashMap::new(),
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

    // ------------------------------------------ Nodes -------------------------------------------

    pub(crate) fn get_or_insert_node(&mut self, path: &str) -> SharedWallpaperNode {
        self.nodes
            .entry(String::from(path))
            .or_insert_with(|| SharedWallpaperNode::new(path, DisplayMode::new(), 1, None))
            .clone()
    }

    pub(crate) fn update_node(&mut self, path: &str, settings: &Settings, period: ParsedTimePeriod) -> SharedWallpaperNode {
        let node = self.get_or_insert_node(path);
        settings.update_node(&mut node.borrow_mut());

        match period {
            ParsedTimePeriod::NotSpecified => {}
            ParsedTimePeriod::Reset        => node.borrow_mut().period = None,
            ParsedTimePeriod::Set(period)  => node.borrow_mut().period = Some(period),
        }

        node
    }

    pub(crate) fn find_actual_nodes(&self) -> Vec<SharedWallpaperNode> {
        let now = Local::now().naive_local();

        let nodes = self.nodes.values()
            .filter(|node| TimePeriod::is_some_and_datetime_in_bounds(&node.borrow().period, &now));

        let group_nodes = self.groups.values()
            .filter(|group| TimePeriod::is_some_and_datetime_in_bounds(&group.period, &now))
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

    pub(crate) fn find_closest_node(&self, path: &str) -> Option<SharedWallpaperNode> {
        self.nodes.values()
            .filter(|node| path.starts_with(node.borrow().path()))
            .max_by_key(|node| node.borrow().path().len())
            .cloned()
    }

    // ---------------------------------------- Wallpapers ----------------------------------------

    pub(crate) fn find_all_child_wallpapers<I, N>(&self, nodes: I) -> (IndexSet<Wallpaper>, Warning)
    where
        I: IntoIterator<Item = N>,
        N: AsRef<SharedWallpaperNode>,
    {
        let mut wallpapers = IndexSet::new();
        let mut warning = Warning::new();

        for rc in nodes {
            let node = rc.as_ref().borrow();
            let path = Path::new(node.path());

            match fs::metadata(path) {
                Ok(metadata) => {
                    self.add_all_child_wallpapers(&mut wallpapers, &mut warning, node, metadata)
                }
                Err(err) => {
                    warning.append_msg_error_path("couldn't get file metadata", &err, node.path())
                }
            }
        }

        (wallpapers, warning)
    }

    fn add_all_child_wallpapers(&self, wallpapers: &mut IndexSet<Wallpaper>, warning: &mut Warning, current_node: Ref<WallpaperNode>, metadata: Metadata) {
        let path = Path::new(current_node.path());

        if metadata.is_file() && ext_matches(path) && check_accessible(path, warning) {
            wallpapers.insert(Wallpaper::from(&*current_node));
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
                        if dir_entry.file_type().is_file()
                            && ext_matches(dir_entry.path())
                            && check_accessible(dir_entry.path(), warning)
                        {
                            let path = dir_entry.path().to_str().unwrap_or_default();

                            wallpapers.insert(Wallpaper::new(
                                String::from(path),
                                self.find_closest_node(path)
                                    .map(|node| node.borrow().mode.clone())
                                    .unwrap_or_else(|| current_node.mode.clone()),
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

    pub(crate) fn run_backend_with_random_wallpaper<'a>(&mut self, wallpapers: &'a IndexSet<Wallpaper>) -> Result<&'a Wallpaper, ActionPerformError> {
        let result = run_backend_with_random_wallpaper_impl(wallpapers);

        self.current_wallpaper_path = result
            .as_ref().ok()
            .map(|wallpaper| wallpaper.path().clone());

        result
    }


    // ------------------------------------------ Other -------------------------------------------

    pub fn get_timeout(&self) -> PollTimeout {
        let now = Local::now().naive_local();

        let time = self
            .nodes
            .values()
            .filter_map(|node| {
                node.borrow().period
                    .as_ref()
                    .map(|period| period.since.clone())
                    .filter(|time| *time > now)
            })
            .min();

        if let Some(time) = time {
            let duration: Result<i32, _> = (time - now).num_milliseconds().try_into();

            match duration {
                Ok(num) => PollTimeout::try_from(num).unwrap(),
                Err(_) => PollTimeout::MAX,
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

static EXTENSIONS: Set<&str> = phf_set! {
    "jpg", "jpeg", "png", "apng", "webp", "gif", "bmp", "tiff", "tif", "ico", "heic", "heif", "avif",
    "mp4", "m4v", "mkv", "avi", "mov", "webm", "flv", "wmv",
};

fn ext_matches(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| EXTENSIONS.contains(ext))
}

fn check_accessible(path: impl AsRef<Path>, warning: &mut Warning) -> bool {
    let path_ref = path.as_ref();

    match File::open(path_ref) {
        Ok(_) => true,

        Err(err) => {
            let path = path_ref.to_str().unwrap_or_default();
            warning.append_msg_error_path("couldn't open file", &err, path);
            false
        }
    }
}


fn run_backend_with_random_wallpaper_impl<'a>(wallpapers: &'a IndexSet<Wallpaper>) -> Result<&'a Wallpaper, ActionPerformError> {
    if wallpapers.is_empty() {
        backend::stop();

        return Err(action_perform_error_localized!(
            "Can't set random wallpaper because there are no valid paths",
            "Не удалось установить случайные обои, так как нет допустимых путей"
        ));
    }

    let wallpaper = &wallpapers[random_range(0..wallpapers.len())];

    match backend::run(wallpaper) {
        Ok(())   => Ok(wallpaper),
        Err(err) => Err(ActionPerformError::from_boxed(err)),
    }
}