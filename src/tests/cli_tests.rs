use clap::CommandFactory;

use crate::cli::Cli;

#[test]
fn verify_cli() {
    <Cli as CommandFactory>::command().debug_assert();
}