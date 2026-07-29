use chrono::{Duration, Local, NaiveDateTime, Timelike};

use crate::server::args_parse_error::ArgsParseError;


pub struct TimePeriod {
    pub since: NaiveDateTime,
    pub until: Option<NaiveDateTime>,
}

impl TimePeriod {
    pub fn new(since: NaiveDateTime, until: Option<NaiveDateTime>) -> Self {
        Self { since, until }
    }
}


pub struct ArgsParseTimePeriod {
    pub since:    Option<NaiveDateTime>,
    pub until:    Option<NaiveDateTime>,
    pub duration: Option<Duration>,
}

impl ArgsParseTimePeriod {
    pub fn new() -> Self {
        Self {
            since:    Option::None,
            until:    Option::None,
            duration: Option::None,
        }
    }

    pub fn set_since(&mut self, since: NaiveDateTime) -> Result<(), ArgsParseError> {
        if self.since.is_some() {
            return Err(ArgsParseError::new("More than one '--since' option specified"));
        }

        self.since = Option::Some(since);
        Ok(())
    }

    pub fn set_until(&mut self, until: NaiveDateTime) -> Result<(), ArgsParseError> {
        if self.until.is_some() {
            return Err(ArgsParseError::new("More than one '--until' option specified"));
        }

        self.until = Option::Some(until);
        Ok(())
    }

    pub fn set_duration(&mut self, duration: Duration) -> Result<(), ArgsParseError> {
        if self.duration.is_some() {
            return Err(ArgsParseError::new("More than one '--duration' option specified"));
        }

        self.duration = Option::Some(duration);
        Ok(())
    }

    pub fn to_time_period(&self) -> Result<TimePeriod, ArgsParseError> {
        let now = Local::now().naive_local()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap();

        if self.until.is_some() && self.duration.is_some() {
            return Err(ArgsParseError::new("The '--until' and '--duration' options cannot be set simultaneously"));
        }

        if self.since.is_some_and(|since| since < now) {
            return Err(ArgsParseError::new("The '--since' time is earlier then now"));
        }

        if self.until.is_some_and(|until| until < now) {
            return Err(ArgsParseError::new("The '--until' time is earlier then now"));
        }

        if  let Some(since) = self.since &&
            let Some(until) = self.until &&
            until <= since {
            
            return Err(ArgsParseError::new("The '--until' time is earlier or equal to '--since'"));
        }

        let since = self.since.unwrap_or(now);

        let until = self.until.or_else(
            || self.duration.map(|dur| since + dur)
        );

        Ok(TimePeriod::new(since, until))
    }
}