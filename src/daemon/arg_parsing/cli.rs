use clap::Parser;
use indoc::indoc;

use crate::daemon::action::ActionResult;
use crate::daemon::arg_parsing::ArgParseError;
use crate::daemon::arg_parsing::action_subcommand::ActionSubcommand;
use crate::daemon::state::State;
use crate::str_localized;

#[derive(Debug, Parser)]
#[command(
    name = "ywpm",
    about = str_localized!(
        "Desktop wallpaper management utility",
        "Утилита управления обоями рабочего стола"
    ),
    long_about = None,
    version,
    term_width = 80,
    disable_help_flag = true,

    help_template = str_localized!(
        indoc! {"
            {about-with-newline}
            Usage: {usage}

            {all-args}
        "},
        indoc! {"
            {about-with-newline}
            Использование: {usage}

            {all-args}
        "}
    )
)]
pub struct Cli {
    #[arg(
        short, long, global = true,
        action = clap::ArgAction::Help,
        help = str_localized!(
            "Show this help",
            "Показать эту справку"
        )
    )]
    help: Option<bool>,

    #[arg(
        short, long, global = true,
        help = str_localized!(
            "Verbose output (for 'get' and 'find-non-fitting')",
            "Подробный вывод (для get и find-non-fitting)"
        )
    )]
    verbose: bool,

    #[arg(
        long, global = true,
        help = str_localized!(
            "Socket for connecting to the server",
            "Сокет для подключения к серверу"
        )
    )]
    socket: Option<String>,

    #[command(subcommand)]
    subcommand: Option<ActionSubcommand>,
}

impl Cli {

    pub fn subcommand(&self) -> &Option<ActionSubcommand> {
        &self.subcommand
    }

    pub fn socket(&self) -> &Option<String> {
        &self.socket
    }

    pub fn canonicalize_paths(&mut self, cwd: &str) -> Result<(), ArgParseError> {
        self.subcommand.as_mut()
            .unwrap()
            .canonicalize_paths(cwd)
    }
    
    pub fn perform_and_update_config(self, state: &mut State) -> ActionResult {
        self.subcommand.unwrap()
            .perform_and_update_config(state, self.verbose)
    }
}
