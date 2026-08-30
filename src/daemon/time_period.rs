use std::fmt;

use chrono::{Locale, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::util::IS_RU;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TimePeriod {
    pub since: NaiveDateTime,
    pub until: NaiveDateTime,
}

impl fmt::Display for TimePeriod {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = if *IS_RU { "%d %B %Y, %H:%M" } else { "%B %d %Y, %H:%M" };
        let locale = if *IS_RU { Locale::ru_RU } else { Locale::default() };

        if self.since.date() == self.until.date() {
            write!(f, "{} - {}",
                self.since.and_utc().format_localized(format, locale),
                self.until.and_utc().format_localized("%H:%M", locale)
            )
        } else {
            write!(f, "{} - {}",
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
        *date_time >= self.since &&
        *date_time <= self.until
    }

    pub fn is_expired(&self, date_time: &NaiveDateTime) -> bool {
        *date_time > self.until
    }
}