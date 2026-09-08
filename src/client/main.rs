use std::error::Error;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::os::unix::net::UnixStream;
use std::process::exit;

use ywpm::format_localized;
use ywpm::reader;
use ywpm::util;
use ywpm::writer;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = match UnixStream::connect(util::get_socket_path()) {
        Ok(stream) => stream,

        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!(
                "{}",
                format_localized!("ywpm-daemon is not running", "ywpm-daemon не запущен")
            );

            exit(1);
        }

        Err(err) => {
            return Err(Box::new(err));
        }
    };

    let args: Vec<String> = std::env::args().collect();

    writer::write_string_vec(&mut BufWriter::new(&mut stream), &args)?;

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
