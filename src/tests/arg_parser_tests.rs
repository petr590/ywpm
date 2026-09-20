use std::error::Error;
use std::{env, fs};

use chrono::{Days, Duration, Local, NaiveDate, Timelike};
use clap::Parser;
use clap::error::ErrorKind;

use crate::assert_is_err;
use crate::daemon::arg_parsing::{ActionSubcommand, Cli, CliTimePeriod, Settings};
use crate::daemon::state::{AlignX, AlignY, DisplayMode, FitMode};

macro_rules! vec_strings {
    ($($item:expr),* $(,)?) => {
        vec![$(String::from($item)),*]
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
        *Cli::parse_from([CMD]).subcommand(),
        None
    );

    assert_eq!(
        Cli::try_parse_from([CMD, "help"]).unwrap_err().kind(),
        ErrorKind::DisplayHelp
    );

    assert_eq!(
        *Cli::parse_from([CMD, "list"]).subcommand(),
        Some(ActionSubcommand::GetNodeList)
    );

    assert_eq!(
        *Cli::parse_from([CMD, "get"]).subcommand(),
        Some(ActionSubcommand::GetCurrentWallpaper)
    );

    assert_eq!(
        *Cli::parse_from([CMD, "random"]).subcommand(),
        Some(ActionSubcommand::SetRandowWallpaper)
    );

    assert_eq!(
        *Cli::parse_from([CMD, "group-list"]).subcommand(),
        Some(ActionSubcommand::GetGroupList)
    );

    assert_eq!(
        *Cli::parse_from([CMD, "set", "--mode", "contain top right", "-r", "3", "/tmp/ywpm-test/a"]).subcommand(),

        Some(ActionSubcommand::SetWallpaper {
            path: String::from("/tmp/ywpm-test/a"),
            settings: Settings {
                recursive_level: Some(3),
                mode: Some(DisplayMode {
                    fit_mode: FitMode::Contain,
                    align_x: AlignX::Right,
                    align_y: AlignY::Top,
                }),
            },
            period: CliTimePeriod { since: None, until: None, duration: None }
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "add", "-m", "top", "./b/c"]).subcommand(),

        Some(ActionSubcommand::AddNodes {
            paths: vec_strings!["./b/c"],
            settings: Settings {
                recursive_level: None,
                mode: Some(DisplayMode {
                    fit_mode: FitMode::Cover,
                    align_x: AlignX::Center,
                    align_y: AlignY::Top,
                }),
            },
            period: CliTimePeriod { since: None, until: None, duration: None }
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "remove", "/tmp", "./b/c", "../x/y/z"]).subcommand(),
        
        Some(ActionSubcommand::RemoveNodes {
            paths: vec_strings!["/tmp", "./b/c", "../x/y/z"]
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "new-group", "chebureki", "/tmp", "../x/y/z"]).subcommand(),

        Some(ActionSubcommand::NewGroup {
            name: String::from("chebureki"),
            paths: vec_strings!["/tmp", "../x/y/z"],
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "get-group", "chebureki"]).subcommand(),
        
        Some(ActionSubcommand::GetGroup {
            name: String::from("chebureki")
        })
    );

    let now = Local::now().naive_local()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap();

    assert_eq!(
        *Cli::parse_from([CMD, "set-group", "-s=now", "-u=tomorrow", "-r=10", "chebureki"]).subcommand(),

        Some(ActionSubcommand::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: Some(10)
            },
            period: CliTimePeriod {
                since: Some(now),
                until: Some((now.date() + Days::new(1)).and_hms_opt(0, 0, 0).unwrap()),
                duration: None,
            }
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "set-group", "-s", "3500-08-08 20:05", "-d=10h", "-r", "65535", "chebureki"]).subcommand(),

        Some(ActionSubcommand::SetGroup {
            name: String::from("chebureki"),
            settings: Settings {
                mode: None,
                recursive_level: Some(65535)
            },
            period: CliTimePeriod {
                since: Some(
                    NaiveDate::from_ymd_opt(3500, 8, 8).unwrap()
                        .and_hms_opt(20, 5, 0).unwrap()
                ),

                duration: Some(Duration::hours(10)),

                until: None,
            }
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "add-to-group", "chebureki", "../x/y/z", "."]).subcommand(),
        Some(ActionSubcommand::AddToGroup {
            name: String::from("chebureki"),
            paths: vec_strings!["../x/y/z", "."],
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "remove-from-group", "chebureki", "../x/y/z", ".", "--", "-x"]).subcommand(),

        Some(ActionSubcommand::RemoveFromGroup {
            name: String::from("chebureki"),
            paths: vec_strings!["../x/y/z", ".", "-x"],
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "clear-group", "chebureki"]).subcommand(),
        Some(ActionSubcommand::ClearGroup {
            name: String::from("chebureki")
        })
    );

    assert_eq!(
        *Cli::parse_from([CMD, "remove-group", "chebureki"]).subcommand(),
        Some(ActionSubcommand::RemoveGroup {
            name: String::from("chebureki")
        })
    );

    Ok(())
}

#[test]
fn parse_args_errors() {
    setup_dirs();

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
