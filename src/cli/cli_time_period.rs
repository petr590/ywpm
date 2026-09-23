use chrono::{Duration, Local, NaiveDateTime, Timelike};
use clap::Args;
use clap::error::ErrorKind;
use indoc::indoc;

use crate::cli::{ParsedTimePeriod, time_parser};
use crate::state::TimePeriod;
use crate::str_localized;


macro_rules! clap_error_localized_str {
    ($kind:expr, $en_msg:expr, $ru_msg:expr) => {
        clap::Error::raw($kind, str_localized!($en_msg, $ru_msg))
    };
}


#[derive(Debug, PartialEq, Args)]
pub struct CliTimePeriod {
    #[arg(
        short, long,
        value_parser = time_parser::parse_time_to_minutes,
        help = str_localized!(
            "Period start (default: now). Format: \"14:00\", \"2026-06-11 12:00\"",
            "Начало периода (по умолчанию: now). Формат: \"14:00\", \"2026-06-11 12:00\""
        )
    )]
    pub(crate) since: Option<NaiveDateTime>,

    #[arg(
        short, long,
        value_parser = time_parser::parse_time_to_minutes,
        conflicts_with = "duration",
        help = str_localized!(
            "Period end. Format: date/time, now, tomorrow",
            "Конец периода. Формат: дата/время, now, tomorrow"
        )
    )]
    pub(crate) until: Option<NaiveDateTime>,

    #[arg(
        short, long,
        value_parser = time_parser::parse_duration,
        conflicts_with = "until",
        help = str_localized!(
            indoc! {"
                Period duration (e.g.: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year, 20:30).
                Months and years are counted as 30 and 365 days, respectively
            "},
            indoc! {"
                Длительность периода (например: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year,
                20:30). Месяцы и годы считаются по 30 и 365 дней соответственно
            "}
        )
    )]
    pub(crate) duration: Option<Duration>,
}

impl CliTimePeriod {
    
    pub(crate) const fn is_none(&self) -> bool {
        self.since.is_none() &&
        self.until.is_none() &&
        self.duration.is_none()
    }

    pub(crate) fn as_parsed_time_period(&self) -> Result<ParsedTimePeriod, clap::Error> {
        if self.is_none() {
            return Ok(ParsedTimePeriod::NotSpecified);
        }

        let now = Local::now().naive_local()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();

        if self.until.is_some() && self.duration.is_some() {
            return Err(clap_error_localized_str!(
                ErrorKind::ArgumentConflict,
                "The '--until' and '--duration' options cannot be set simultaneously",
                "Параметры '--until' и '--duration' не могут быть заданы одновременно"
            ));
        }

        let since = self.since.unwrap_or(now);

        let until = self.until
            .or_else(|| self.duration.map(|dur| since + dur))
            .ok_or_else(|| clap_error_localized_str!(
                ErrorKind::MissingRequiredArgument,
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

    fn check_time_bounds(&self, now: NaiveDateTime) -> Result<(), clap::Error> {
        if self.since.is_some_and(|since| since < now) {
            return Err(clap_error_localized_str!(
                ErrorKind::InvalidValue,
                "The '--since' time is earlier then now",
                "Время '--since' раньше текущего момента"
            ));
        }

        if self.until.is_some_and(|until| until < now) {
            return Err(clap_error_localized_str!(
                ErrorKind::InvalidValue,
                "The '--until' time is earlier then now",
                "Время '--until' раньше текущего момента"
            ));
        }

        if  let Some(since) = self.since &&
            let Some(until) = self.until &&
            until < since
        {
            return Err(clap_error_localized_str!(
                ErrorKind::InvalidValue,
                "The '--until' time is earlier to '--since'",
                "Время '--until' раньше времени '--since'"
            ));
        }

        Ok(())
    }
}
