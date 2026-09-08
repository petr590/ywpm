use std::error::Error;
use std::{env, fs};

use chrono::{Days, Local, NaiveDate, Timelike};

use crate::assert_is_err;
use crate::daemon::action::Action;
use crate::daemon::arg_parsing::{ParsedTimePeriod, parse_args};
use crate::daemon::state::{AlignX, AlignY, DisplayMode, FitMode, Settings, TimePeriod};
use crate::util;

macro_rules! vec_strings {
    ($($item:expr),* $(,)?) => {
        vec![$(String::from($item)),*]
    };
}

macro_rules! vec_canonicalized_paths {
    ($($item:expr),* $(,)?) => {
        vec![$(util::canonicalize_path($item)?),*]
    };
}

const CMD: &str = "/usr/bim/ywpm";

fn setup_dirs() {
    fs::create_dir_all("/tmp/ywpm-test/a/b/c").unwrap();
    fs::create_dir_all("/tmp/ywpm-test/x/y/z").unwrap();
    fs::create_dir_all("/tmp/ywpm-test/a/-x").unwrap();
    env::set_current_dir("/tmp/ywpm-test/a").unwrap();
}

#[test]
fn parse_args_correctly() -> Result<(), Box<dyn Error>> {
    setup_dirs();

    assert_eq!(
        parse_args(&vec_strings![CMD, "help", "--mode", "center"])?,
        Action::Help
    );
    assert_eq!(parse_args(&vec_strings![CMD, "list"])?, Action::GetNodeList);
    assert_eq!(
        parse_args(&vec_strings![CMD, "get"])?,
        Action::GetCurrentWallpaper
    );
    assert_eq!(
        parse_args(&vec_strings![CMD, "random"])?,
        Action::SetRandowWallpaper
    );
    assert_eq!(
        parse_args(&vec_strings![CMD, "group-list"])?,
        Action::GetGroupList
    );

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "--mode",
            "contain top right",
            "-r",
            "3",
            "set",
            "/tmp/ywpm-test/a"
        ])?,
        Action::SetWallpaper {
            path: util::canonicalize_path("/tmp/ywpm-test/a")?,
            settings: Settings {
                recursive_level: Some(3),
                mode: Some(DisplayMode {
                    fit_mode: FitMode::Contain,
                    align_x: AlignX::Right,
                    align_y: AlignY::Top,
                }),
            },
            period: ParsedTimePeriod::NotSpecified
        }
    );

    assert_eq!(
        parse_args(&vec_strings![CMD, "add", "-m", "top", "./b/c"])?,
        Action::AddNodes {
            paths: vec_canonicalized_paths!["./b/c"],
            settings: Settings {
                recursive_level: None,
                mode: Some(DisplayMode {
                    fit_mode: FitMode::Cover,
                    align_x: AlignX::Center,
                    align_y: AlignY::Top,
                }),
            }
        }
    );

    assert_eq!(
        parse_args(&vec_strings![CMD, "remove", "/tmp", "./b/c", "../x/y/z"])?,
        Action::RemoveNodes {
            paths: vec_canonicalized_paths!["/tmp", "./b/c", "../x/y/z"]
        }
    );

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "new-group",
            "chebureki",
            "/tmp",
            "../x/y/z"
        ])?,
        Action::NewGroup {
            name: String::from("chebureki"),
            paths: vec_canonicalized_paths!["/tmp", "../x/y/z"],
        }
    );

    assert_eq!(
        parse_args(&vec_strings![CMD, "get-group", "chebureki"])?,
        Action::GetGroup {
            name: String::from("chebureki")
        }
    );

    let now = Local::now()
        .naive_local()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "set-group",
            "-s=now",
            "-u=tomorrow",
            "-r=10",
            "chebureki"
        ])?,
        Action::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: Some(10)
            },
            period: ParsedTimePeriod::Set(TimePeriod::new(
                now,
                (now.date() + Days::new(1)).and_hms_opt(0, 0, 0).unwrap()
            ))
        }
    );

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "set-group",
            "-s",
            "3500-08-08 20:05",
            "-d=10h",
            "-r",
            "65535",
            "chebureki"
        ])?,
        Action::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: Some(65535)
            },
            period: ParsedTimePeriod::Set(TimePeriod::new(
                NaiveDate::from_ymd_opt(3500, 8, 8)
                    .unwrap()
                    .and_hms_opt(20, 5, 0)
                    .unwrap(),
                NaiveDate::from_ymd_opt(3500, 8, 9)
                    .unwrap()
                    .and_hms_opt(6, 5, 0)
                    .unwrap(),
            ))
        }
    );

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "add-to-group",
            "chebureki",
            "../x/y/z",
            "."
        ])?,
        Action::AddToGroup {
            name: String::from("chebureki"),
            paths: vec_canonicalized_paths!["../x/y/z", "."],
        }
    );

    assert_eq!(
        parse_args(&vec_strings![
            CMD,
            "remove-from-group",
            "chebureki",
            "../x/y/z",
            ".",
            "--",
            "-x"
        ])?,
        Action::RemoveFromGroup {
            name: String::from("chebureki"),
            paths: vec_canonicalized_paths!["../x/y/z", ".", "-x"],
        }
    );

    assert_eq!(
        parse_args(&vec_strings![CMD, "clear-group", "chebureki"])?,
        Action::ClearGroup {
            name: String::from("chebureki")
        }
    );

    assert_eq!(
        parse_args(&vec_strings![CMD, "remove-group", "chebureki"])?,
        Action::RemoveGroup {
            name: String::from("chebureki")
        }
    );

    Ok(())
}

#[test]
fn parse_args_errors() {
    setup_dirs();

    assert_is_err!(parse_args(&vec_strings![CMD]));
    assert_is_err!(parse_args(&vec_strings![CMD, ""]));
    assert_is_err!(parse_args(&vec_strings![CMD, "-x"]));
    assert_is_err!(parse_args(&vec_strings![CMD, "-m", "center", "get"]));
    assert_is_err!(parse_args(&vec_strings![CMD, "-u", "23:00", "remove", "."]));
    assert_is_err!(parse_args(&vec_strings![CMD, "remove"]));
    assert_is_err!(parse_args(&vec_strings![CMD, "-u", "123", "set", "."]));
    assert_is_err!(parse_args(&vec_strings![CMD, "add", "chebureki", "-m"]));
    assert_is_err!(parse_args(&vec_strings![CMD, "add", "chebureki", "-m=abc"]));
}
