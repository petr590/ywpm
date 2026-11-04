use chrono::{DateTime, Days, Local, NaiveDateTime, NaiveTime, Timelike};

use crate::arg_parse_error_localized;
use crate::cli::cli_duration_parser;
use crate::cli::error::ArgParseError;

pub(crate) fn parse_time_to_minutes(input: &str) -> Result<NaiveDateTime, ArgParseError> {
    Ok(parse_time(input)?
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap())
}

fn parse_time(input: &str) -> Result<NaiveDateTime, ArgParseError> {
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "now" => return Ok(Local::now().naive_local()),

        "tomorrow" => {
            let tomorrow = Local::now().date_naive() + Days::new(1);
            return Ok(tomorrow.and_hms_opt(0, 0, 0).unwrap());
        }

        _ => {}
    }

    if input.starts_with('+') {
        let duration = cli_duration_parser::parse_num_and_unit(&input[1..])?
            .ok_or_else(|| cli_duration_parser::invalid_duration_error(&input))?;

        return Ok(Local::now().naive_local() + duration);
    }

    if input.contains('-') && input.contains(':') {
        let formats = [
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
        ];

        for fmt in formats {
            if let Ok(date_time) = NaiveDateTime::parse_from_str(&input, fmt) {
                return Ok(date_time);
            }
        }
    }

    if input.contains(':') {
        let formats = ["%H:%M", "%H:%M:%S", "%H:%M:%S%.f"];

        for fmt in formats {
            if let Ok(time) = NaiveTime::parse_from_str(&input, fmt) {
                return Ok(Local::now().date_naive().and_time(time));
            }
        }
    }

    if let Ok(date_time) = DateTime::parse_from_rfc3339(&input) {
        return Ok(date_time.naive_local());
    }

    if let Ok(date_time) = DateTime::parse_from_rfc2822(&input) {
        return Ok(date_time.naive_local());
    }

    Err(arg_parse_error_localized!(
        "Invalid date/time value: '{input}'",
        "Недопустимое значение даты/времени: '{input}'"
    ))
}