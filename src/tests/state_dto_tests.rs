use std::env;
use std::fs::{self, File};
use std::io::{self, ErrorKind};
use std::num::NonZeroU16;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chrono::{Days, Local};

use crate::config::Config;
use crate::core::Warning;
use crate::nzu16;
use crate::state::{
    Alignment, DisplayMode, FitMode, SharedWallpaperNode, State, TimePeriod, Wallpaper, WallpaperGroup,
};
use crate::tests::util;

#[test]
fn state_serializes() {
    let mut state = State::new(create_some_config());

    let wallpaper = add_wallpaper(
        &mut state,
        "/a/b/c.jpg",
        DisplayMode::new(),
        nzu16!(1),
        None
    ).clone();

    add_wallpaper(
        &mut state,
        "/a/b/c/",
        DisplayMode::new_with_fields(FitMode::Stretch, Alignment::Center),
        nzu16!(1),
        Some(TimePeriod::new(
            Local::now().naive_local(),
            Local::now().naive_local() + Days::new(10),
        )),
    );

    state.groups
        .entry(String::from("some_group"))
        .or_insert_with(WallpaperGroup::new)
        .add(wallpaper);

    let content = yaml_serde::to_string(&state.as_dto()).unwrap();
    println!("{content}");
}

fn add_wallpaper(state: &mut State, path: impl Into<PathBuf>, mode: DisplayMode, recursive_level: NonZeroU16, period: Option<TimePeriod>) -> &mut SharedWallpaperNode {
    let path = path.into();

    state.nodes
        .entry(path.clone())
        .or_insert(SharedWallpaperNode::new(path, mode, recursive_level, period))
}

fn create_test_files() {
    env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();

    let test_dir = *util::TEST_DIR;
    let unaccessible_file = test_dir.join("unaccessible.jpeg");

    fs::create_dir_all(test_dir).unwrap();

    for entry in fs::read_dir("src/tests/data").unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), test_dir.join(entry.file_name())).unwrap();
    }

    File::create(test_dir.join("f.txt")).unwrap();
    File::create(test_dir.join("f")).unwrap();
    
    match File::create(&unaccessible_file) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::PermissionDenied => {}
        Err(err) => panic!("{err}")
    }

    let result = match File::open(unaccessible_file) {
        Ok(file) => Ok(file),
        Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(err),
        Err(err) => panic!("{err}"),
    };

    if let Ok(file) = result {
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o000);
        file.set_permissions(permissions).unwrap();
    }

    create_symlink_if_not_exists(test_dir, "", "recurse-dir").unwrap();
    create_symlink_if_not_exists(test_dir, "1x1.png", "symlink.png").unwrap();
    create_symlink_if_not_exists(test_dir, "unexisting.png", "broken_symlink.png").unwrap();
}

fn create_symlink_if_not_exists(test_dir: impl AsRef<Path>, original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<(), io::Error> {
    let original = test_dir.as_ref().join(original);
    let link = test_dir.as_ref().join(link);

    match symlink(original, link) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

#[test]
fn state_returns_all_wallpapers() {
    create_test_files();

    let mode = DisplayMode::new();
    let period = Some(TimePeriod::new(
        Local::now().naive_local(),
        Local::now().naive_local() + Days::new(10),
    ));

    let mut state = State::new(create_some_config());
    let test_dir = *util::TEST_DIR;

    add_wallpaper(&mut state, test_dir, mode.clone(), nzu16!(2), period);

    let mut warning = Warning::new();

    let wallpapers = state.find_all_child_wallpapers(state.nodes.values(), &mut warning);

    println!("{wallpapers:?}");

    assert_eq!(wallpapers.len(), 4);

    assert!(wallpapers.contains(&Wallpaper::new(test_dir.join("1x1.png"),     mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new(test_dir.join("1x1.jpg"),     mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new(test_dir.join("1x1.webp"),    mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new(test_dir.join("symlink.png"), mode.clone())));

    assert!(!wallpapers.contains(&Wallpaper::new(test_dir.join("unaccessible.jpeg"),  mode.clone())));
    assert!(!wallpapers.contains(&Wallpaper::new(test_dir.join("broken_symlink.png"), mode.clone())));
}

fn create_some_config() -> Config {
	Config::new("/tmp/state.yaml", Some(Rc::from("")), Some(Rc::from("")))
}