use chrono::{Duration, Local, NaiveDateTime, Timelike};

use crate::arg_parse_error_localized;
use crate::daemon::arg_parsing::error::ArgParseError;
use crate::daemon::time_period::TimePeriod;

#[derive(PartialEq)]
pub struct ArgParseTimePeriod {
    pub since:    Option<NaiveDateTime>,
    pub until:    Option<NaiveDateTime>,
    pub duration: Option<Duration>,
}

impl ArgParseTimePeriod {
    pub const fn new() -> Self {
        Self {
            since:    None,
            until:    None,
            duration: None,
        }
    }

    pub const fn is_none(&self) -> bool {
        self.since.is_none() &&
        self.until.is_none() &&
        self.duration.is_none()
    }

    pub fn set_since(&mut self, since: NaiveDateTime) -> Result<(), ArgParseError> {
        if self.since.is_some() {
            return Err(arg_parse_error_localized!(
                "More than one '--since' option specified",
                "Указано более одного параметра '--since'"
            ));
        }

        self.since = Some(since);
        Ok(())
    }

    pub fn set_until(&mut self, until: NaiveDateTime) -> Result<(), ArgParseError> {
        if self.until.is_some() {
            return Err(arg_parse_error_localized!(
                "More than one '--until' option specified",
                "Указано более одного параметра '--until'"
            ));
        }

        self.until = Some(until);
        Ok(())
    }

    pub fn set_duration(&mut self, duration: Duration) -> Result<(), ArgParseError> {
        if self.duration.is_some() {
            return Err(arg_parse_error_localized!(
                "More than one '--duration' option specified",
                "Указано более одного параметра '--duration'"
            ));
        }

        self.duration = Some(duration);
        Ok(())
    }

    pub fn as_time_period_opt(&self) -> Result<Option<TimePeriod>, ArgParseError> {
        if self.is_none() {
            return Ok(None);
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
            until <= since {
            
            return Err(arg_parse_error_localized!(
                "The '--until' time is earlier or equal to '--since'",
                "Время '--until' раньше или равно времени '--since'"
            ));
        }

        let since = self.since.unwrap_or(now);

        let until = self.until
            .or_else(|| self.duration.map(|dur| since + dur))
            .ok_or_else(|| arg_parse_error_localized!(
                "If the '--since' option is set, then one of '--until' or '--duration' options must also be set",
                "Если указана опция '--since', то также должна быть указана одна из опций '--until' или '--duration'"
            ))?;

        Ok(Some(TimePeriod::new(since, until)))
    }
}