use std::env;
use std::error::Error;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::os::unix::net::UnixStream;
use std::process::exit;

use clap::{Arg, Command};
use ywpm::format_localized;
use ywpm::reader;
use ywpm::util;
use ywpm::writer;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = match UnixStream::connect(get_socket_path()) {
        Ok(stream) => stream,

        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!("{}", format_localized!(
                "ywpmd is not running. Try: systemctl --user start ywpmd.service",
                "ywpmd не запущен. Попробуйте: systemctl --user start ywpmd.service"
            ));

            exit(1);
        }

        Err(err) => {
            return Err(Box::new(err));
        }
    };

    {
        let mut buf_writer = BufWriter::new(&mut stream);
        writer::write_string(&mut buf_writer, env::current_dir()?.to_str().unwrap_or_default())?;
        writer::write_string_vec(&mut buf_writer, &std::env::args().collect())?;
    }

    let action_result = reader::read_response(&mut BufReader::new(&mut stream))?;

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

fn get_socket_path() -> String {
    let matches = Command::new("ywpm")
        .ignore_errors(true) 
        .arg(
            Arg::new("socket")
                .long("socket")
                .num_args(1)
                .required(false),
        )
        .try_get_matches_from(env::args())
        .unwrap_or_else(|_| {
            Command::new("ywpm").get_matches()
        });
    
    matches.get_one::<String>("socket")
        .cloned()
        .unwrap_or_else(util::get_socket_path)
}