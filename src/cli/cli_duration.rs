use std::ops::Add;

use chrono::{Duration, Months, NaiveDateTime};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CliDuration {
    Seconds(i64),
    Months(u32),
    Years(u32),
}

impl Add<CliDuration> for NaiveDateTime {
    type Output = NaiveDateTime;

    fn add(self, duration: CliDuration) -> Self::Output {
        match duration {
            CliDuration::Seconds(seconds) => self + Duration::seconds(seconds),
            CliDuration::Months(months)   => self.checked_add_months(Months::new(months)).expect("`NaiveDateTime + CliDuration::Months` overflowed"),
            CliDuration::Years(years)     => self.checked_add_months(Months::new(years * 12)).expect("`NaiveDateTime + CliDuration::Years` overflowed"),
        }
    }
}

impl CliDuration {
    pub fn seconds (value: i64) -> Self { Self::Seconds(value) }
    pub fn minutes (value: i64) -> Self { Self::Seconds(value * 60) }
    pub fn hours   (value: i64) -> Self { Self::Seconds(value * (60 * 60)) }
    pub fn days    (value: i64) -> Self { Self::Seconds(value * (60 * 60 * 24)) }
    pub fn weeks   (value: i64) -> Self { Self::Seconds(value * (60 * 60 * 24 * 7)) }
    pub fn months  (value: u32) -> Self { Self::Months(value) }
    pub fn years   (value: u32) -> Self { Self::Years(value) }
}