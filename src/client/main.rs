use std::path::Path;
use std::{env, io};
use std::error::Error;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, IsTerminal};
use std::os::unix::net::UnixStream;
use std::process::exit;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use ywpm::cli::{ActionSubcommand, Cli};
use ywpm::{format_localized, localized};
use ywpm::util::{self, ReadWriteResult};

fn main() -> Result<(), Box<dyn Error>> {

    CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    let socket_path = cli.socket_path()
            .clone()
            .unwrap_or_else(util::get_socket_path);


    let mut stream = connect_to_socket(socket_path)?;

    let confirmed = cli.skip_confirmation() ||
            !io::stdin().is_terminal() ||
            confirm_removing_and_clearing(cli.subcommand());

    if confirmed {
        write_args(&mut stream)?;
        read_response(&mut stream)?;
    } else {
        util::write_bool(&mut stream, false)?;
    }

    Ok(())
}

fn connect_to_socket(socket_path: impl AsRef<Path>) -> Result<UnixStream, io::Error> {
    let result = UnixStream::connect(socket_path);

    if let Err(ref io_err) = result &&
        io_err.kind() == ErrorKind::NotFound
    {
        eprintln!("{}", format_localized!(
            "ywpmd is not running. Try: systemctl --user start ywpmd.service",
            "ywpmd не запущен. Попробуйте: systemctl --user start ywpmd.service"
        ));

        exit(1);
    }

    result
}

fn confirm_removing_and_clearing(subcommand: &ActionSubcommand) -> bool {

    match subcommand {
        ActionSubcommand::RemoveNodes     { .. } |
        ActionSubcommand::RemoveFromGroup { .. } => confirm(
            "Do you really want to remove wallpapers from database?",
            "Вы действительно хотите удалить обои из базы данных?"
        ),

        ActionSubcommand::RemoveGroup { .. } => confirm(
            "Do you really want to remove group from database?",
            "Вы действительно хотите удалить группу из базы данных?"
        ),

        ActionSubcommand::ClearNodes { .. } => confirm(
            "Do you really want to remove all data from database?",
            "Вы действительно хотите удалить все данные из базы данных?"
        ),

        ActionSubcommand::ClearGroup { .. } => confirm(
            "Do you really want to remove all group's data?",
            "Вы действительно хотите удалить все данные группы?"
        ),

        _ => true,
    }
}

fn confirm(en_msg: &str, ru_msg: &str) -> bool {
    let message = localized!(en_msg, ru_msg);

    eprint!("{message} [Y/n]: ");

    for result in io::stdin().lock().lines() {

        if let Ok(line) = result {
            match line.trim().to_lowercase().as_str() {
                "y" | "yes" | "" => {
                    return true;
                }

                "n" | "no" => {
                    eprintln!("{}", localized!("Aborted", "Прервано"));
                    return false;
                }

                _ => {}
            }
        }

        eprint!("{message} [Y/n]: ");
    }

    false
}

fn write_args(stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(stream);
    util::write_bool(&mut writer, true)?;
    util::write_string(&mut writer, env::current_dir()?.to_str().unwrap_or_default())?;
    util::write_string_vec(&mut writer, &env::args().collect())?;
    util::write_u16(&mut writer, util::get_terminal_width())?;
    Ok(())
}

fn read_response(stream: &mut UnixStream) -> ReadWriteResult<()> {
    let action_result = util::read_response(&mut BufReader::new(stream))?;

    match action_result {
        Ok(success) => {
            print!("{success}");
            Ok(())
        }

        Err(error) => {
            eprintln!("{error}");
            exit(1);
        }
    }
}