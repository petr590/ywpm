use chrono::{Duration, Local, NaiveDateTime, Timelike};

use crate::arg_parse_error_localized;
use crate::daemon::arg_parsing::{ArgParseError, ParsedTimePeriod};
use crate::daemon::state::TimePeriod;

#[derive(PartialEq)]
pub struct ArgParseTimePeriod {
    pub since:    Option<NaiveDateTime>,
    pub until:    Option<NaiveDateTime>,
    pub duration: Option<Duration>,
}

impl ArgParseTimePeriod {
    pub const fn new() -> Self {
        Self {
            since: None,
            until: None,
            duration: None,
        }
    }

    pub const fn is_none(&self) -> bool {
        self.since.is_none() && self.until.is_none() && self.duration.is_none()
    }

    pub fn set_since(&mut self, since: NaiveDateTime) -> Result<(), ArgParseError> {
        if self.since.is_some() {
            return Err(more_than_one_option_specified_error("--since"));
        }

        self.since = Some(since);
        Ok(())
    }

    pub fn set_until(&mut self, until: NaiveDateTime) -> Result<(), ArgParseError> {
        if self.until.is_some() {
            return Err(more_than_one_option_specified_error("--until"));
        }

        self.until = Some(until);
        Ok(())
    }

    pub fn set_duration(&mut self, duration: Duration) -> Result<(), ArgParseError> {
        if self.duration.is_some() {
            return Err(more_than_one_option_specified_error("--duration"));
        }

        self.duration = Some(duration);
        Ok(())
    }

    pub fn as_parsed_time_period(&self) -> Result<ParsedTimePeriod, ArgParseError> {
        if self.is_none() {
            return Ok(ParsedTimePeriod::NotSpecified);
        }

        let now = Local::now().naive_local()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();

        if self.until.is_some() && self.duration.is_some() {
            return Err(arg_parse_error_localized!(
                "The '--until' and '--duration' options cannot be set simultaneously",
                "Параметры '--until' и '--duration' не могут быть заданы одновременно"
            ));
        }

        let since = self.since.unwrap_or(now);

        let until = self.until
            .or_else(|| self.duration.map(|dur| since + dur))
            .ok_or_else(|| arg_parse_error_localized!(
                "If the '--since' option is set, then one of '--until' or '--duration' options must also be set",
                "Если указана опция '--since', то также должна быть указана одна из опций '--until' или '--duration'"
            ))?;

        self.check_time_bounds(now)?;

        Ok(if since == until {
            ParsedTimePeriod::Reset
        } else {
            ParsedTimePeriod::Set(TimePeriod::new(since, until))
        })
    }

    fn check_time_bounds(&self, now: NaiveDateTime) -> Result<(), ArgParseError> {
        if self.since.is_some_and(|since| since < now) {
            return Err(arg_parse_error_localized!(
                "The '--since' time is earlier then now",
                "Время '--since' раньше текущего момента"
            ));
        }

        if self.until.is_some_and(|until| until < now) {
            return Err(arg_parse_error_localized!(
                "The '--until' time is earlier then now",
                "Время '--until' раньше текущего момента"
            ));
        }

        if  let Some(since) = self.since &&
            let Some(until) = self.until &&
            until < since
        {
            return Err(arg_parse_error_localized!(
                "The '--until' time is earlier to '--since'",
                "Время '--until' раньше времени '--since'"
            ));
        }

        Ok(())
    }
}

fn more_than_one_option_specified_error(opt_name: &str) -> ArgParseError {
    arg_parse_error_localized!(
        "More than one '{opt_name}' option specified",
        "Указано более одного параметра '{opt_name}'"
    )
}
