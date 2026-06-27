use std::os::unix::net::UnixStream;
use std::io::{BufReader, BufWriter};
use std::error::Error;
use std::process::exit;

use ywpm::reader::read_response;
use ywpm::writer::write_string_vec;
use ywpm::util::get_socket_path;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = UnixStream::connect(get_socket_path())?;
    let args: Vec<String> = std::env::args().collect();
    
    write_string_vec(&mut BufWriter::new(&mut stream), &args)?;

    let (is_ok, message) = read_response(&mut BufReader::new(&mut stream))?;

    if is_ok {
        println!("{message}");
        Ok(())
    } else {
        eprintln!("{message}");
        exit(1);
    }
}