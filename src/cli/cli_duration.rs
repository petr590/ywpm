use std::ops::Add;

use chrono::{Duration, Months, NaiveDateTime};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum CliDuration {
    Minutes(i64),
    Months(u32),
    Years(u32),
}

impl Add<CliDuration> for NaiveDateTime {
    type Output = NaiveDateTime;

    fn add(self, duration: CliDuration) -> Self::Output {
        match duration {
            CliDuration::Minutes(minutes) => self + Duration::minutes(minutes),
            CliDuration::Months(months)   => self.checked_add_months(Months::new(months)).expect("`NaiveDateTime + CliDuration::Months` overflowed"),
            CliDuration::Years(years)     => self.checked_add_months(Months::new(years * 12)).expect("`NaiveDateTime + CliDuration::Years` overflowed"),
        }
    }
}

impl CliDuration {
    pub(crate) const ZERO: CliDuration = CliDuration::Minutes(0);

    pub(crate) fn minutes (value: i64) -> Self { Self::Minutes(value) }
    pub(crate) fn hours   (value: i64) -> Self { Self::Minutes(value * 60) }
    pub(crate) fn days    (value: i64) -> Self { Self::Minutes(value * (60 * 24)) }
    pub(crate) fn weeks   (value: i64) -> Self { Self::Minutes(value * (60 * 24 * 7)) }
    pub(crate) fn months  (value: u32) -> Self { Self::Months(value) }
    pub(crate) fn years   (value: u32) -> Self { Self::Years(value) }
}