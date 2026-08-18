use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TimePeriod {
    pub since: NaiveDateTime,
    pub until: NaiveDateTime,
}

impl TimePeriod {
    pub fn new(since: NaiveDateTime, until: NaiveDateTime) -> Self {
        Self { since, until }
    }
}