use clap::{ArgAction, Parser, ValueHint};
use indoc::indoc;

use crate::cli::ArgParseError;
use crate::cli::action_subcommand::ActionSubcommand;
use crate::localized;

#[derive(Debug, Parser)]
#[command(
    name = "ywpm",
    about = localized!(
        "Desktop wallpaper management utility",
        "Утилита управления обоями рабочего стола"
    ),
    long_about = None,
    version,
    term_width = 80,
    disable_help_flag = true,

    help_template = localized!(
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
        action = ArgAction::Help,
        help = localized!(
            "Show this help",
            "Показать эту справку"
        )
    )]
    help: Option<bool>,

    #[arg(
        short = 'y',
        global = true,
        help = localized!(
            "Do not ask for confirmation before removing wallpapers/groups from config",
            "Не спрашивать подтверждение перед удалением обоев/групп из конфига"
        )
    )]
    skip_confirmation: bool,

    #[arg(
        short, long, global = true,
        help = localized!(
            "Verbose output (for 'get' and 'find-non-fitting')",
            "Подробный вывод (для get и find-non-fitting)"
        )
    )]
    verbose: bool,

    #[arg(
        long = "socket",
        global = true,
        value_hint = ValueHint::FilePath,
        help = localized!(
            "Socket for connecting to the server",
            "Сокет для подключения к серверу"
        )
    )]
    socket_path: Option<String>,

    #[command(subcommand)]
    subcommand: ActionSubcommand,
}

impl Cli {

    pub fn skip_confirmation(&self) -> bool {
        self.skip_confirmation
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn subcommand(&self) -> &ActionSubcommand {
        &self.subcommand
    }

    pub fn subcommand_move(self) -> ActionSubcommand {
        self.subcommand
    }

    pub fn socket_path(&self) -> &Option<String> {
        &self.socket_path
    }

    pub fn canonicalize_paths(&mut self, cwd: &str) -> Result<(), ArgParseError> {
        self.subcommand.canonicalize_paths(cwd)
    }
}
