use clap::{Args, Command, FromArgMatches};

use crate::cli::CliTimePeriod;
use crate::state::TimePeriod;

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedTimePeriod {
    NotSpecified,
    Reset,
    Set(TimePeriod),
}

impl Args for ParsedTimePeriod {
    fn augment_args(cmd: Command) -> Command {
        CliTimePeriod::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        CliTimePeriod::augment_args_for_update(cmd)
    }
}

impl FromArgMatches for ParsedTimePeriod {

    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let cli_period = CliTimePeriod::from_arg_matches(matches)?;
        Ok(cli_period.as_parsed_time_period()?)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
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