use chrono::{DateTime, Days, Local, NaiveDateTime, NaiveTime, Timelike};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::arg_parse_error_localized;
use crate::cli::cli_duration::CliDuration;
use crate::cli::error::ArgParseError;

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

pub(crate) fn parse_time_to_minutes(input: &str) -> Result<NaiveDateTime, ArgParseError> {
    Ok(parse_time(input)?
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap())
}

static ZERO_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?-u)^0+$").unwrap());
static NUM_AND_UNIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?-u)^(\d+)\s*(\w+)$").unwrap());
static TIME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?-u)^(\d+):(\d{1,2})$").unwrap());

pub(crate) fn parse_duration(input: &str) -> Result<CliDuration, ArgParseError> {
    let input = input.trim().to_lowercase();

    if ZERO_REGEX.is_match(&input) {
        return Ok(CliDuration::Seconds(0));
    }

    if let Some(caps) = NUM_AND_UNIT_REGEX.captures(&input) {
        let value: i64 = caps[1].parse().map_err(|_| {
            arg_parse_error_localized!(
                "Invalid number: {}",
                "Недопустимое число: {}",
                &caps[1]
            )
        })?;

        let unit = &caps[2];

        return Ok(match unit {
            "m" | "min" | "mins" | "minute" | "minutes" => CliDuration::minutes(value),
            "h" | "hr"  | "hrs"  | "hour"   | "hours"   => CliDuration::hours(value),

            "d" | "day"   | "days"           => CliDuration::days(value),
            "w" | "week"  | "weeks"          => CliDuration::weeks(value),
                  "month" | "months"         => CliDuration::months(i64_to_u32(value, unit)?),
            "y" | "yr"    | "year" | "years" => CliDuration::years(i64_to_u32(value, unit)?),

            _ => {
                return Err(arg_parse_error_localized!(
                    "Unknown unit: {unit}",
                    "Неизвестная единица измерения: {unit}",
                ));
            }
        });
    }

    if let Some(caps) = TIME_REGEX.captures(&input) {
        let hours: i64 = caps[1].parse().map_err(|_| {
            arg_parse_error_localized!(
                "Invalid hours: {}",
                "Недопустимое количество часов: {}",
                &caps[1]
            )
        })?;

        let minutes: i64 = caps[2].parse().ok()
            .filter(|mins| (0..60).contains(mins))
            .ok_or_else(|| {
                arg_parse_error_localized!(
                    "Invalid minutes: {}",
                    "Недопустимое количество минут: {}",
                    &caps[2]
                )
            })?;

        return Ok(CliDuration::minutes(hours * 60 + minutes));
    }

    Err(arg_parse_error_localized!(
        "Invalid duration value: {input}",
        "Недопустимое значение длительности: {input}"
    ))
}


fn i64_to_u32(value: i64, unit: &str) -> Result<u32, ArgParseError> {
    u32::try_from(value)
        .map_err(|_| arg_parse_error_localized!(
            "Period is too big: {value} {unit}",
            "Период слишком большой: {value} {unit}"
        ))
}