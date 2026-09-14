use crate::daemon::state::TimePeriod;

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedTimePeriod {
    NotSpecified,
    Reset,
    Set(TimePeriod),
}

impl ParsedTimePeriod {
    pub fn unwrap(&self) -> &TimePeriod {
        match self {
            Self::Set(period) => period,
            Self::NotSpecified => {
                panic!("called `unwrap()` on a `ParsedTimePeriod::NotSpecified` value")
            }
            Self::Reset => panic!("called `unwrap()` on a `ParsedTimePeriod::Reset` value"),
        }
    }
}