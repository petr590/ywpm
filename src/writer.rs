use std::io::{self, Write};


pub fn write_string_vec(stream: &mut impl Write, vec: &Vec<String>) -> io::Result<()> {
    stream.write_all(&(vec.len() as u32).to_be_bytes())?;

    for s in vec {
        stream.write_all(&(s.len() as u32).to_be_bytes())?;
        stream.write_all(s.as_bytes())?;
    }

    Ok(())
}

pub fn write_response(writer: &mut impl Write, is_ok: bool, string: &str) -> io::Result<()> {
    writer.write_all(&[if is_ok { 1 } else { 0 }])?;
    writer.write_all(&(string.len() as u32).to_be_bytes())?;
    writer.write_all(&string.as_bytes())?;
    Ok(())
}