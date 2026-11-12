use std::sync::LazyLock;

use regex::Regex;

use crate::arg_parse_error_localized;
use crate::cli::CliDuration;
use crate::cli::error::ArgParseError;


static ZERO_REGEX:         LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?-u)^0+$").unwrap());
static NUM_AND_UNIT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?-u)^(\d+)\s*(\w+)$").unwrap());
static TIME_REGEX:         LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?-u)^(\d+):(\d{1,2})$").unwrap());


pub(crate) fn parse_duration(input: &str) -> Result<CliDuration, ArgParseError> {
    let input = input.trim().to_lowercase();

    if ZERO_REGEX.is_match(&input) {
        return Ok(CliDuration::ZERO);
    }

    if let Some(duration) = parse_num_and_unit(&input)? {
        return Ok(duration);
    }

    if let Some(duration) = parse_minutes_hours(&input)? {
        return Ok(duration);
    }

    Err(invalid_duration_error(&input))
}

pub(super) fn parse_num_and_unit(input: &str) -> Result<Option<CliDuration>, ArgParseError> {

    if let Some(caps) = NUM_AND_UNIT_REGEX.captures(input) {
        let value: i64 = caps[1].parse().map_err(|_| {
            arg_parse_error_localized!(
                "Invalid number: {}",
                "Недопустимое число: {}",
                &caps[1]
            )
        })?;

        let unit = &caps[2];

        let duration = match unit {
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
        };

        Ok(Some(duration))
    } else {
        Ok(None)
    }
}

fn parse_minutes_hours(input: &str) -> Result<Option<CliDuration>, ArgParseError> {

    if let Some(caps) = TIME_REGEX.captures(input) {
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

        Ok(Some(CliDuration::minutes(hours * 60 + minutes)))
    } else {
        Ok(None)
    }
}


fn i64_to_u32(value: i64, unit: &str) -> Result<u32, ArgParseError> {
    u32::try_from(value)
        .map_err(|_| arg_parse_error_localized!(
            "Period is too big: {value} {unit}",
            "Период слишком большой: {value} {unit}"
        ))
}


pub(super) fn invalid_duration_error(input: &str) -> ArgParseError {
    arg_parse_error_localized!(
        "Invalid duration value: {input}",
        "Недопустимое значение длительности: {input}"
    )
}