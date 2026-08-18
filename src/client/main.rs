use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::io::{BufReader, BufWriter};
use std::error::Error;
use std::process::exit;

use ywpm::format_localized;
use ywpm::reader;
use ywpm::writer;
use ywpm::util;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = match UnixStream::connect(util::get_socket_path()) {
        Ok(stream) => stream,

        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!("{}", format_localized!(
                "ywpm-daemon is not running",
                "ywpm-daemon не запущен"
            ));

            exit(1);
        }

        Err(err) => {
            return Err(Box::new(err));
        }
    };


    let args: Vec<String> = std::env::args().collect();
    
    writer::write_string_vec(&mut BufWriter::new(&mut stream), &args)?;

    let (is_ok, message) = reader::read_response(&mut BufReader::new(&mut stream))?;

    if is_ok {
        print!("{message}");

        if !message.is_empty() && !message.ends_with("\n") {
            println!();
        }

        Ok(())

    } else {
        eprint!("{message}");

        if !message.is_empty() && !message.ends_with("\n") {
            eprintln!();
        }

        exit(1);
    }
}