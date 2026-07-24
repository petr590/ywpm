use std::error::Error;
use std::io::ErrorKind;
use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;

use serde::{Deserialize, Serialize};

use crate::server::state::State;
use crate::server::display_mode::{DisplayMode, FitMode};
use crate::server::wallpaper_node::WallpaperNode;

#[test]
fn state_serializes() {
    let mut state = State::new();
    let wallpaper = state.add_wallpaper("/a/b/c.jpg", DisplayMode::new(), 1).clone();
    state.add_wallpaper("/a/b/c/", DisplayMode::new().fit_mode(FitMode::Stretch), 1);
    state.add_group("some_group").add(wallpaper);

    let content = yaml_serde::to_string(&state.as_dto()).unwrap();
    println!("{content}");
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
        Err(err) => panic!("{err}")
    };

    if let Ok(file) = result {
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o000);
        file.set_permissions(permissions).unwrap();
    }

    create_symlink_if_not_exists("/tmp/ywpm-test",       "/tmp/ywpm-test/recurse-dir");
    create_symlink_if_not_exists("/tmp/ywpm-test/a.png", "/tmp/ywpm-test/a_ref.png");
    create_symlink_if_not_exists("nonexistent.png",      "/tmp/ywpm-test/broken_symlink.png");
}

fn create_symlink_if_not_exists(original: &str, link: &str) {
    match symlink(original, link) {
        Ok(()) => (),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => (),
        Err(err) => panic!("{err}")
    }
}


#[test]
fn state_returns_all_wallpapers() {
    create_test_files();

    let mode = DisplayMode::new();
    let recursive_level = 2;

    let mut state = State::new();
    state.add_wallpaper("/tmp/ywpm-test/", mode.clone(), recursive_level.clone());

    let (nodes, _) = state.all_wallpapers();

    println!("{nodes:?}");

    assert_eq!(nodes.len(), 4);
    assert!(nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/a.png", mode.clone(), recursive_level.clone())));
    assert!(nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/b.jpg", mode.clone(), recursive_level.clone())));
    assert!(nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/c.webp", mode.clone(), recursive_level.clone())));
    assert!(nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/a_ref.png", mode.clone(), recursive_level.clone())));

    assert!(!nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/unaccessible.jpeg", mode.clone(), recursive_level.clone())));
    assert!(!nodes.contains(&WallpaperNode::new("/tmp/ywpm-test/broken_symlink.png", mode.clone(), recursive_level.clone())));
}


#[derive(Serialize, Deserialize)]
struct DisplayModeProxy {
    #[serde(with = "crate::server::display_mode_format")]
    mode: DisplayMode
}


#[test]
fn display_mode_serializes() -> Result<(), Box<dyn Error>> {
    let mode = DisplayMode::new();
    assert_eq!(mode.to_string(), "cover center center");

    let proxy = DisplayModeProxy { mode };
    assert_eq!(yaml_serde::to_string(&proxy)?.trim(), "mode: cover center center");

    Ok(())
}