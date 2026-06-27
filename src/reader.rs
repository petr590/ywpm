use std::io::{self, Read};
use std::error::Error;

use crate::server::package_error::PackageError;


pub fn read_string_vec(stream: &mut impl Read) -> Result<Vec<String>, Box<dyn Error>> {
    let size = read_size(stream)?;

    if size == 0 {
        return Err(Box::new(PackageError::EmptyPackage));
    }

    let mut vec = Vec::new();
    vec.reserve_exact(size);

    for _ in 0..size {
        let len = read_size(stream)?;
        let data = read_data(stream, len)?;
        vec.push(String::from_utf8(data)?);
    }

    Ok(vec)
}

pub fn read_response(stream: &mut impl Read) -> Result<(bool, String), Box<dyn Error>> {
    let is_ok = read_bool(stream)?;
    let size = read_size(stream)?;
    let data = read_data(stream, size)?;
    Ok((is_ok, String::from_utf8(data)?))
}


fn read_bool(stream: &mut impl Read) -> io::Result<bool> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(buf[0] != 0)
}

fn read_size(stream: &mut impl Read) -> io::Result<usize> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf) as usize)
}

fn read_data(stream: &mut impl Read, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}