use clap::CommandFactory;

use crate::daemon::arg_parsing::Cli;

#[test]
fn verify_cli() {
    <Cli as CommandFactory>::command().debug_assert();
}