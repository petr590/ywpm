use chrono::{Days, Local, NaiveDate, Timelike};

use crate::cli::{ArgParseError, CliDuration, parse_duration, parse_time_to_minutes};
use crate::assert_is_err;

#[test]
fn check_time_parses_normally() -> Result<(), ArgParseError> {
    assert_eq!(
        parse_time_to_minutes("now")?,
        Local::now().naive_local()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap()
    );

    assert_eq!(
        parse_time_to_minutes("tomorrow")?,
        Local::now().naive_local()
            .date()
            .checked_add_days(Days::new(1)).unwrap()
            .and_hms_opt(0, 0, 0).unwrap()
    );

    let date_time = Local::now().naive_local()
        .date()
        .and_hms_opt(20, 8, 0).unwrap();

    assert_eq!(parse_time_to_minutes("20:08")?,              date_time);
    assert_eq!(parse_time_to_minutes("20:08:32")?,           date_time);
    assert_eq!(parse_time_to_minutes("20:08:32.123457890")?, date_time);

    let date_time = NaiveDate
        ::from_ymd_opt(2026, 7, 27).unwrap()
        .and_hms_opt(20, 8, 0).unwrap();

    assert_eq!(parse_time_to_minutes("2026-07-27 20:08")?,                date_time);
    assert_eq!(parse_time_to_minutes("2026-07-27 20:08:32")?,             date_time);
    assert_eq!(parse_time_to_minutes("2026-07-27 20:08:32.123457890")?,   date_time);
    assert_eq!(parse_time_to_minutes("2026-07-27T20:08:32.123457890Z")?,  date_time);
    assert_eq!(parse_time_to_minutes("27 Jul 2026 20:08:32 +0000")?,      date_time);
    assert_eq!(parse_time_to_minutes("MON, 27 jul 2026 20:08:32 +0000")?, date_time);
    Ok(())
}

#[test]
fn check_time_parses_fails() {
    assert!(parse_time_to_minutes("2026-07-27").is_err());
    assert!(parse_time_to_minutes("2026-07-27 25:08").is_err());
    assert!(parse_time_to_minutes("2026-7-27T20:08:32.123457890Z").is_err());
}

#[test]
fn check_duration_parses_normally() -> Result<(), ArgParseError> {
    assert_eq!(parse_duration("2m")?,   CliDuration::minutes(2));
    assert_eq!(parse_duration("3h")?,   CliDuration::hours(3));
    assert_eq!(parse_duration("100d")?, CliDuration::days(100));
    assert_eq!(parse_duration("100w")?, CliDuration::weeks(100));
    assert_eq!(parse_duration("5y")?,   CliDuration::days(5 * 365));

    assert_eq!(parse_duration("2mins")?,      CliDuration::minutes(2));
    assert_eq!(parse_duration("3   hrs")?,    CliDuration::hours(3));
    assert_eq!(parse_duration("100 days")?,   CliDuration::days(100));
    assert_eq!(parse_duration("125 months")?, CliDuration::days(125 * 30));
    assert_eq!(parse_duration("5\t years")?,  CliDuration::days(5 * 365));

    assert_eq!(parse_duration("32:59")?, CliDuration::minutes(32 * 60 + 59));

    Ok(())
}

#[test]
fn check_duration_parse_fails() {
    assert_is_err!(parse_duration("32:64"));
}
