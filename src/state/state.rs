use std::cell::Ref;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::{self, File, Metadata};
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDateTime};
use indexmap::IndexSet;
use itertools::Itertools;
use nix::NixPath;
use nix::poll::PollTimeout;
use phf::{Set, phf_set};
use rand::random_range;
use walkdir::WalkDir;

use crate::daemon::service::MediaType;
use crate::{action_perform_error_localized, nzu16};
use crate::cli::{ParsedTimePeriod, Settings};
use crate::config::Config;
use crate::core::ActionPerformError;
use crate::daemon::{backend, service};
use crate::core::Warning;
use crate::state::{
    DisplayMode, SharedWallpaperNode, StateDto, TimePeriod, Wallpaper, WallpaperGroup, WallpaperNode,
};

#[derive(Debug, Clone)]
pub struct State {
	config: Config,
    pub current_wallpaper_path: Option<PathBuf>,
    pub nodes: HashMap<PathBuf, SharedWallpaperNode>,
    pub groups: HashMap<String, WallpaperGroup>,
}

impl State {
    // ----------------------------------------- Creation -----------------------------------------

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn path(&self) -> &Path {
		self.config.state_path()
	}

    pub fn new(config: Config) -> Self {
        Self {
			config,
            current_wallpaper_path: None,
            nodes: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    pub fn from_dto(config: Config, dto: StateDto) -> Self {
        let nodes = dto.nodes
            .into_iter()
            .map(|node| {
                let path = PathBuf::from(node.borrow().path());
                (path, node)
            })
            .collect();

        let groups = dto.groups.into_iter()
            .map(|(name, group_dto)| (name, WallpaperGroup::from_dto(group_dto, &nodes)))
            .collect();

        Self {
			config,
            current_wallpaper_path: dto.current_wallpaper_path.clone(),
            nodes,
            groups,
        }
    }

    pub fn as_dto(&self) -> StateDto {
        let mut nodes = self.nodes.iter()
                .map(|(_path, node)| node.clone())
                .collect::<Vec<SharedWallpaperNode>>();
        
        nodes.sort_by(|node1, node2| {
            node1.borrow().path()
                .cmp(node2.borrow().path())
        });

        StateDto {
            current_wallpaper_path: self.current_wallpaper_path.clone(),
            nodes,
            groups: self.groups.iter()
                .map(|(name, group)| (name.clone(), group.as_dto()))
                .collect(),
        }
    }

    // ------------------------------------------ Nodes -------------------------------------------

    pub(crate) fn get_or_insert_node(&mut self, path: &Path) -> SharedWallpaperNode {
        match self.nodes.get(path) {
            Some(node) => node.clone(),
            None => {
                let node = SharedWallpaperNode::new(path, DisplayMode::new(), nzu16!(1), None);
                self.nodes.insert(path.to_path_buf(), node.clone());
                node
            }
        }
    }

    pub(crate) fn update_node(&mut self, path: &Path, settings: &Settings, period: ParsedTimePeriod) -> SharedWallpaperNode {
        let node = self.get_or_insert_node(path);
        settings.update_node(&mut node.borrow_mut());

        match period {
            ParsedTimePeriod::NotSpecified => {}
            ParsedTimePeriod::Reset        => node.borrow_mut().period = None,
            ParsedTimePeriod::Set(period)  => node.borrow_mut().period = Some(period),
        }

        node
    }

    /// Ищет все актуальные `SharedWallpaperNode`. Актуальные - ознчает, что узел подходит по периоду времени.
    /// Если ни один узел не подходит по периоду времени, то возвращается список всех `SharedWallpaperNode`.
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

    pub(crate) fn find_closest_node(&self, path: &Path) -> Option<SharedWallpaperNode> {
        self.nodes.values()
            .filter(|node| path.starts_with(node.borrow().path()))
            .max_by_key(|node| node.borrow().path().len())
            .cloned()
    }

    // ---------------------------------------- Wallpapers ----------------------------------------

    pub(crate) fn find_all_child_wallpapers<I, N>(&self, nodes: I, warning: &mut Warning) -> IndexSet<Wallpaper>
    where
        I: IntoIterator<Item = N>,
        N: AsRef<SharedWallpaperNode>,
    {
        let mut wallpapers = IndexSet::new();

        for rc in nodes {
            let node = rc.as_ref().borrow();

            match fs::metadata(node.path()) {
                Ok(metadata) => {
                    self.add_all_child_wallpapers(&mut wallpapers, warning, node, metadata)
                }
                Err(err) => {
                    warning.append_msg_error_path("couldn't get file metadata", &err, node.path())
                }
            }
        }

        wallpapers
    }

    fn add_all_child_wallpapers(&self, wallpapers: &mut IndexSet<Wallpaper>, warning: &mut Warning, current_node: Ref<WallpaperNode>, metadata: Metadata) {
        let current_path = current_node.path();
        let matcher = self.config.media_type_matcher();

        if metadata.is_file()
            && ext_matches(current_path)
            && check_accessible(current_path, warning)
        {
            let media_type = service::get_media_type(current_path);

            if media_type.as_ref().is_ok_and(|typ| matcher(*typ)) {
                wallpapers.insert(Wallpaper::from(&*current_node));

            } else {
                let media_type_str = match media_type {
                    Ok(MediaType::Image) => "image",
                    Ok(MediaType::Video) => "video",
                    Err(_) => "file with unknown type",
                };

                let _ = writeln!(warning,
                    "Warning: couldn't find command for {}: '{}'",
                    media_type_str, current_path.display()
                );
            }

            return;
        }

        if metadata.is_dir() {
            let walk_dir = WalkDir::new(current_path)
                .follow_links(true)
                .same_file_system(false)
                .max_depth(current_node.recursive_level.get() as usize);

            for entry in walk_dir {
                match entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        
                        if dir_entry.file_type().is_file()
                            && ext_matches(path)
                            && check_accessible(path, warning)
                            && service::get_media_type(path).is_ok_and(&matcher)
                        {
                            wallpapers.insert(Wallpaper::new(
                                path,
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
        let result = run_backend_with_random_wallpaper_impl(&self.config, wallpapers);

        self.current_wallpaper_path = result
            .as_ref().ok()
            .map(|wallpaper| wallpaper.path().to_path_buf());

        result
    }


    // ------------------------------------------ Other -------------------------------------------
    pub fn get_timeout(&self) -> PollTimeout {
        let now = Local::now().naive_local();

        let time = self.nodes
            .values()
            .filter_map(|node| {
                node.borrow().period
                    .as_ref()
                    .map(|period| period.since.clone())
                    .filter(|since| *since > now)
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

    pub fn normalize(&mut self) -> &mut Self {
        self.clear_expired_periods();
        self.clear_redundant_paths();
        self
    }

    fn clear_expired_periods(&mut self) {
        let now = Local::now().naive_local();

        for node in self.nodes.values_mut() {
            clear_period_if_expired(&mut node.borrow_mut().period, &now);
        }

        for group in self.groups.values_mut() {
            clear_period_if_expired(&mut group.period, &now);
        }
    }

    fn clear_redundant_paths(&mut self) {
        if self.nodes.len() < 2 {
            return;
        }

        let mut paths = self.nodes.keys()
                .map(|p| p.as_path())
                .collect::<Vec<&Path>>();
        
        paths.sort();

        let mut redudant: Vec<PathBuf> = Vec::new();
        let mut base_path = paths[0];

        for &path in &paths[1..] {
            if self.is_redudant(path, base_path) {
                redudant.push(path.to_path_buf());
            } else {
                base_path = path;
            }
        }

        for path in redudant {
            self.nodes.remove(&path);
        }
    }

    fn is_redudant(&self, child_path: &Path, parent_path: &Path) -> bool {
        if !child_path.starts_with(parent_path) {
            return false;
        }

        let child  = self.nodes.get(child_path).unwrap().borrow();
        let parent = self.nodes.get(parent_path).unwrap().borrow();

        child.mode == parent.mode && (
            child.period == parent.period ||
            child.period.is_none()
        ) && (
            parent.recursive_level > child.recursive_level ||
            child_path.is_file()
        )
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
    let path = path.as_ref();

    match File::open(path) {
        Ok(_) => true,

        Err(err) => {
            let path = path.to_str().unwrap_or_default();
            warning.append_msg_error_path("couldn't open file", &err, path);
            false
        }
    }
}


fn run_backend_with_random_wallpaper_impl<'a>(config: &Config, wallpapers: &'a IndexSet<Wallpaper>) -> Result<&'a Wallpaper, ActionPerformError> {
    if wallpapers.is_empty() {
        backend::stop();

        return Err(action_perform_error_localized!(
            "Can't set random wallpaper because there are no valid paths",
            "Не удалось установить случайные обои, так как нет допустимых путей"
        ));
    }

    let wallpaper = &wallpapers[random_range(0..wallpapers.len())];

    match backend::run(config, wallpaper) {
        Ok(())   => Ok(wallpaper),
        Err(err) => Err(ActionPerformError::new(err.to_string())),
    }
}
