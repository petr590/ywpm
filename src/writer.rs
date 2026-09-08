use std::io::{self, Write};

use crate::daemon::action::ActionSuccess;

pub fn write_string_vec(writer: &mut impl Write, vec: &Vec<String>) -> io::Result<()> {
    writer.write_all(&(vec.len() as u64).to_be_bytes())?;

    for s in vec {
        writer.write_all(&(s.len() as u64).to_be_bytes())?;
        writer.write_all(s.as_bytes())?;
    }

    Ok(())
}

pub fn write_ok(writer: &mut impl Write, success: ActionSuccess) -> io::Result<()> {
    writer.write_all(&[0])?;
    write_string(writer, &success.message)?;
    write_string(writer, success.warning.message())?;
    Ok(())
}

pub fn write_error(writer: &mut impl Write, message: &str) -> io::Result<()> {
    writer.write_all(&[1])?;
    write_string(writer, message)?;
    Ok(())
}

fn write_string(writer: &mut impl Write, string: &str) -> io::Result<()> {
    writer.write_all(&(string.len() as u64).to_be_bytes())?;
    writer.write_all(&string.as_bytes())?;
    Ok(())
}
