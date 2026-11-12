use std::error::Error;
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::{env, fs};

use chrono::{Days, Local, NaiveDate, Timelike};
use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;

use crate::cli::{ActionSubcommand, Cli, ParsedTimePeriod, Settings};
use crate::assert_is_err;
use crate::state::{Alignment, DisplayMode, FitMode, TimePeriod};
use crate::tests::util;

macro_rules! vec_paths {
    ($($item:expr),* $(,)?) => {
        vec![$(PathBuf::from($item)),*]
    };
}

const CMD: &str = "/usr/bim/ywpm";

#[test]
fn verify_cli() {
    <Cli as CommandFactory>::command().debug_assert();
}

#[test]
fn parse_args_correctly() -> Result<(), Box<dyn Error>> {
    setup_dirs();

    assert_eq!(
        Cli::try_parse_from([CMD, "help"]).unwrap_err().kind(),
        ErrorKind::DisplayHelp
    );

    assert_eq!(
        Cli::parse_from([CMD, "list"]).subcommand,
        ActionSubcommand::GetNodeList
    );

    assert_eq!(
        Cli::parse_from([CMD, "get"]).subcommand,
        ActionSubcommand::GetCurrentWallpaper
    );

    assert_eq!(
        Cli::parse_from([CMD, "random"]).subcommand,
        ActionSubcommand::SetRandowWallpaper
    );

    assert_eq!(
        Cli::parse_from([CMD, "group-list"]).subcommand,
        ActionSubcommand::GetGroupList
    );

    let path = util::TEST_DIR.join("a");

    assert_eq!(
        Cli::parse_from([CMD, "set", "--mode", "stretch top contain right", "-r", "3", &path.to_string_lossy()]).subcommand,

        ActionSubcommand::SetWallpaper {
            path: path,
            settings: Settings {
                recursive_level: NonZeroU16::new(3),
                mode: Some(DisplayMode {
                    fit_mode:  FitMode::Contain,
                    alignment: Alignment::Right,
                }),
            },
            period: ParsedTimePeriod::NotSpecified
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "add", "-m", "top", "./b/c"]).subcommand,

        ActionSubcommand::AddNodes {
            paths: vec_paths!["./b/c"],
            settings: Settings {
                recursive_level: None,
                mode: Some(DisplayMode {
                    fit_mode:  FitMode::Cover,
                    alignment: Alignment::Top,
                }),
            },
            period: ParsedTimePeriod::NotSpecified
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "remove", "/tmp", "./b/c", "../x/y/z"]).subcommand,
        
        ActionSubcommand::RemoveNodes {
            paths: vec_paths!["/tmp", "./b/c", "../x/y/z"]
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "new-group", "chebureki", "/tmp", "../x/y/z"]).subcommand,

        ActionSubcommand::NewGroup {
            name: String::from("chebureki"),
            paths: vec_paths!["/tmp", "../x/y/z"],
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "get-group", "chebureki"]).subcommand,
        
        ActionSubcommand::GetGroup {
            name: String::from("chebureki")
        }
    );

    let now = Local::now().naive_local()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap();

    assert_eq!(
        Cli::parse_from([CMD, "set-group", "-s=now", "-u=tomorrow", "-r=10", "chebureki"]).subcommand,

        ActionSubcommand::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: NonZeroU16::new(10)
            },
            period: ParsedTimePeriod::Set(TimePeriod {
                since: now,
                until: (now.date() + Days::new(1)).and_hms_opt(0, 0, 0).unwrap(),
            })
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "set-group", "-s", "3500-08-08 20:05", "-d=10h", "-r", "65535", "chebureki"]).subcommand,

        ActionSubcommand::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: NonZeroU16::new(65535)
            },
            period: ParsedTimePeriod::Set(TimePeriod {
                since: NaiveDate::from_ymd_opt(3500, 8, 8).unwrap()
                        .and_hms_opt(20, 5, 0).unwrap(),

                until: NaiveDate::from_ymd_opt(3500, 8, 9).unwrap()
                        .and_hms_opt(6, 5, 0).unwrap(),
            })
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "add-to-group", "chebureki", "../x/y/z", "."]).subcommand,
        ActionSubcommand::AddToGroup {
            name: String::from("chebureki"),
            paths: vec_paths!["../x/y/z", "."],
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "remove-from-group", "chebureki", "../x/y/z", ".", "--", "-x"]).subcommand,

        ActionSubcommand::RemoveFromGroup {
            name: String::from("chebureki"),
            paths: vec_paths!["../x/y/z", ".", "-x"],
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "clear-group", "chebureki"]).subcommand,
        ActionSubcommand::ClearGroup {
            name: String::from("chebureki")
        }
    );

    assert_eq!(
        Cli::parse_from([CMD, "remove-group", "chebureki"]).subcommand,
        ActionSubcommand::RemoveGroup {
            name: String::from("chebureki")
        }
    );

    Ok(())
}

#[test]
fn parse_args_errors() {
    setup_dirs();

    assert_is_err!(Cli::try_parse_from([CMD]));
    assert_is_err!(Cli::try_parse_from([CMD, ""]));
    assert_is_err!(Cli::try_parse_from([CMD, "-x"]));
    assert_is_err!(Cli::try_parse_from([CMD, "get", "-m", "center"]));
    assert_is_err!(Cli::try_parse_from([CMD, "-m", "center", "get"]));
    assert_is_err!(Cli::try_parse_from([CMD, "-u", "23:00", "remove", "."]));
    assert_is_err!(Cli::try_parse_from([CMD, "remove"]));
    assert_is_err!(Cli::try_parse_from([CMD, "-u", "123", "set", "."]));
    assert_is_err!(Cli::try_parse_from([CMD, "add", "chebureki", "-m"]));
    assert_is_err!(Cli::try_parse_from([CMD, "add", "chebureki", "-m=abc"]));
}

fn setup_dirs() {
    let test_dir = *util::TEST_DIR;
    fs::create_dir_all(test_dir.join("a/b/c")).unwrap();
    fs::create_dir_all(test_dir.join("x/y/z")).unwrap();
    fs::create_dir_all(test_dir.join("a/-x")).unwrap();
    env::set_current_dir(test_dir.join("a")).unwrap();
}