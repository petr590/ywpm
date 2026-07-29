use chrono::{DateTime, Days, Duration, Local, NaiveDateTime, NaiveTime, Timelike};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{args_parse_error_format, server::args_parse_error::ArgsParseError};

fn parse_time(input: &str) -> Result<NaiveDateTime, ArgsParseError> {
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "now" => {
            return Ok(Local::now().naive_local())
        },

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
        let time_formats = [
            "%H:%M",
            "%H:%M:%S",
            "%H:%M:%S%.f",
        ];

        for fmt in time_formats {
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


    Err(args_parse_error_format!("Invalid date/time value: {}", input))
}


pub fn parse_time_to_minutes(input: &str) -> Result<NaiveDateTime, ArgsParseError> {
    Ok(parse_time(input)?
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap())
}


static NUM_AND_UNIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)\s*(\w+)$").unwrap());
static TIME_REGEX:         Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+):(\d{1,2})$").unwrap());

pub fn parse_duration(input: &str) -> Result<Duration, ArgsParseError> {
    let input = input.trim().to_lowercase();

    if input == "tomorrow" {
        return Ok(Duration::days(1));
    }

    if let Some(caps) = NUM_AND_UNIT_REGEX.captures(&input) {
        let value: i64 = caps[1].parse()
            .map_err(|_| args_parse_error_format!("Invalid number: {}", &caps[1]))?;


        let unit = &caps[2];

        return Ok(match unit {
            "m" | "min" | "mins" | "minute" | "minutes" => Duration::minutes(value),
            "h" | "hr"  | "hrs"  | "hour"   | "hours"   => Duration::hours(value),

            "d" | "day"   | "days"           => Duration::days(value),
            "w" | "week"  | "weeks"          => Duration::days(value * 7),
                  "month" | "months"         => Duration::days(value * 30),
            "y" | "yr"    | "year" | "years" => Duration::days(value * 365),

            _ => return Err(args_parse_error_format!("Unknown unit: {unit}")),
        });
    }

    if let Some(caps) = TIME_REGEX.captures(&input) {
        let hours: i64 = caps[1].parse()
            .map_err(|_| args_parse_error_format!("Invalid hours: {}", &caps[1]))?;

        let minutes: i64 = caps[2].parse().ok()
            .filter(|m| (0..60).contains(m))
            .ok_or_else(|| args_parse_error_format!("Invalid minutes: {}", &caps[2]))?;

        return Ok(Duration::hours(hours) + Duration::minutes(minutes));
    }

    Err(args_parse_error_format!("Cannot parse duration: {}", input))
}