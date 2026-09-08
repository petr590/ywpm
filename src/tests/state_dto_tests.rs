use std::fs::{self, File};
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use chrono::{Days, Local};

use crate::daemon::state::{
    DisplayMode, FitMode, SharedWallpaperNode, State, TimePeriod, Wallpaper, WallpaperGroup,
};

#[test]
fn state_serializes() {
    let mut state = State::new();
    let wallpaper = add_wallpaper(&mut state, "/a/b/c.jpg", DisplayMode::new(), 1, None).clone();

    add_wallpaper(
        &mut state,
        "/a/b/c/",
        DisplayMode::new().fit_mode(FitMode::Stretch),
        1,
        Some(TimePeriod::new(
            Local::now().naive_local(),
            Local::now().naive_local() + Days::new(10),
        )),
    );

    state
        .groups
        .entry(String::from("some_group"))
        .or_insert_with(WallpaperGroup::new)
        .add(wallpaper);

    let content = yaml_serde::to_string(&state.as_dto()).unwrap();
    println!("{content}");
}

fn add_wallpaper(state: &mut State, path: impl Into<String>, mode: DisplayMode, recursive_level: u16, period: Option<TimePeriod>) -> &mut SharedWallpaperNode {
    let path = path.into();

    state
        .nodes
        .entry(path.clone())
        .or_insert(SharedWallpaperNode::new(
            path,
            mode,
            recursive_level,
            period,
        ))
}

fn create_test_files() {
    const UNACCESSIBLE_FILE: &str = "/tmp/ywpm-test/unaccessible.jpeg";

    const TEST_FILES: [&str; 6] = [
        "/tmp/ywpm-test/a.png",
        "/tmp/ywpm-test/b.jpg",
        "/tmp/ywpm-test/c.webp",
        "/tmp/ywpm-test/d.txt",
        "/tmp/ywpm-test/e",
        UNACCESSIBLE_FILE,
    ];

    fs::create_dir_all("/tmp/ywpm-test/").unwrap();

    for file in TEST_FILES {
        if !fs::exists(file).unwrap() {
            File::create(file).unwrap();
        }
    }

    let result = match File::open(UNACCESSIBLE_FILE) {
        Ok(file) => Ok(file),
        Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(err),
        Err(err) => panic!("{err}"),
    };

    if let Ok(file) = result {
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o000);
        file.set_permissions(permissions).unwrap();
    }

    create_symlink_if_not_exists("/tmp/ywpm-test", "/tmp/ywpm-test/recurse-dir");
    create_symlink_if_not_exists("/tmp/ywpm-test/a.png", "/tmp/ywpm-test/a_ref.png");
    create_symlink_if_not_exists("nonexistent.png", "/tmp/ywpm-test/broken_symlink.png");
}

fn create_symlink_if_not_exists(original: &str, link: &str) {
    match symlink(original, link) {
        Ok(()) => (),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => (),
        Err(err) => panic!("{err}"),
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

    let mut state = State::new();
    add_wallpaper(&mut state, "/tmp/ywpm-test/", mode.clone(), 2, period);

    let (wallpapers, _) = state.find_all_child_wallpapers(state.nodes.values());

    println!("{wallpapers:?}");

    assert_eq!(wallpapers.len(), 4);

    assert!(wallpapers.contains(&Wallpaper::new("/tmp/ywpm-test/a.png", mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new("/tmp/ywpm-test/b.jpg", mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new("/tmp/ywpm-test/c.webp", mode.clone())));
    assert!(wallpapers.contains(&Wallpaper::new("/tmp/ywpm-test/a_ref.png", mode.clone())));

    assert!(!wallpapers.contains(&Wallpaper::new(
        "/tmp/ywpm-test/unaccessible.jpeg",
        mode.clone()
    )));
    assert!(!wallpapers.contains(&Wallpaper::new(
        "/tmp/ywpm-test/broken_symlink.png",
        mode.clone()
    )));
}
