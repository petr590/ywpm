use std::fmt;

use chrono::{Local, Locale, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::localized;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TimePeriod {
    pub since: NaiveDateTime,
    pub until: NaiveDateTime,
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = localized!(
            "%B %-d, %Y, %H:%M",
            "%-d %B %Y, %H:%M"
        );

        let locale = localized!(
            Locale::default(),
            Locale::ru_RU
        );

        if self.since.date() == self.until.date() {
            write!(
                f, "{} - {}",
                self.since.and_utc().format_localized(format, locale),
                self.until.and_utc().format_localized("%H:%M", locale)
            )
        } else {
            write!(
                f, "{} - {}",
                self.since.and_utc().format_localized(format, locale),
                self.until.and_utc().format_localized(format, locale)
            )
        }
    }
}

impl TimePeriod {
    pub fn new(since: NaiveDateTime, until: NaiveDateTime) -> Self {
        Self { since, until }
    }

    pub fn is_datetime_in_bounds(&self, date_time: &NaiveDateTime) -> bool {
        *date_time >= self.since && *date_time <= self.until
    }

    pub fn is_expired(&self, date_time: &NaiveDateTime) -> bool {
        *date_time > self.until
    }

    pub fn is_some_and_datetime_in_bounds(period: &Option<TimePeriod>, date_time: &NaiveDateTime) -> bool {
        period.as_ref()
            .is_some_and(|period| period.is_datetime_in_bounds(date_time))
    }

    pub fn is_none_or_now(period: &Option<TimePeriod>) -> bool {
        let now = Local::now().naive_local();

        period.as_ref()
            .is_none_or(|period| period.is_datetime_in_bounds(&now))
    }

    pub(crate) fn opt_to_string(period: &Option<TimePeriod>) -> String {
        match period {
            Some(period) => period.to_string(),
            None => String::from("none"),
        }
    }
}
