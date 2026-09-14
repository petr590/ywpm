use chrono::{Duration, Local, NaiveDateTime, Timelike};
use clap::Args;

use crate::action_perform_error_localized;
use crate::daemon::action::ActionPerformError;
use crate::daemon::arg_parsing::{ParsedTimePeriod, time_parser};
use crate::daemon::state::TimePeriod;

#[derive(Debug, PartialEq, Args)]
pub struct CliTimePeriod {
    #[arg(
        short, long,
        value_parser = time_parser::parse_time_to_minutes
    )]
    pub(crate) since: Option<NaiveDateTime>,

    #[arg(
        short, long,
        value_parser = time_parser::parse_time_to_minutes,
        conflicts_with = "duration",
    )]
    pub(crate) until: Option<NaiveDateTime>,

    #[arg(
        short, long,
        value_parser = time_parser::parse_duration,
        conflicts_with = "until",
    )]
    pub(crate) duration: Option<Duration>,
}

impl CliTimePeriod {
    
    pub(crate) const fn is_none(&self) -> bool {
        self.since.is_none() &&
        self.until.is_none() &&
        self.duration.is_none()
    }

    pub(crate) fn as_parsed_time_period(&self) -> Result<ParsedTimePeriod, ActionPerformError> {
        if self.is_none() {
            return Ok(ParsedTimePeriod::NotSpecified);
        }

        let now = Local::now().naive_local()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();

        if self.until.is_some() && self.duration.is_some() {
            return Err(action_perform_error_localized!(
                "The '--until' and '--duration' options cannot be set simultaneously",
                "Параметры '--until' и '--duration' не могут быть заданы одновременно"
            ));
        }

        let since = self.since.unwrap_or(now);

        let until = self.until
            .or_else(|| self.duration.map(|dur| since + dur))
            .ok_or_else(|| action_perform_error_localized!(
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

    fn check_time_bounds(&self, now: NaiveDateTime) -> Result<(), ActionPerformError> {
        if self.since.is_some_and(|since| since < now) {
            return Err(action_perform_error_localized!(
                "The '--since' time is earlier then now",
                "Время '--since' раньше текущего момента"
            ));
        }

        if self.until.is_some_and(|until| until < now) {
            return Err(action_perform_error_localized!(
                "The '--until' time is earlier then now",
                "Время '--until' раньше текущего момента"
            ));
        }

        if  let Some(since) = self.since &&
            let Some(until) = self.until &&
            until < since
        {
            return Err(action_perform_error_localized!(
                "The '--until' time is earlier to '--since'",
                "Время '--until' раньше времени '--since'"
            ));
        }

        Ok(())
    }
}
