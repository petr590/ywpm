use std::io::Write;

use crate::core::ActionSuccess;
use crate::util::{ReadWriteError, ReadWriteResult};

pub fn write_string_vec(writer: &mut impl Write, vec: &Vec<String>) -> ReadWriteResult<()> {
    write_all(writer, &(vec.len() as u64).to_be_bytes())?;

    for s in vec {
        write_all(writer, &(s.len() as u64).to_be_bytes())?;
        write_all(writer, s.as_bytes())?;
    }

    Ok(())
}

pub fn write_string(writer: &mut impl Write, string: &str) -> ReadWriteResult<()> {
    write_all(writer, &(string.len() as u64).to_be_bytes())?;
    write_all(writer, &string.as_bytes())?;
    Ok(())
}

pub fn write_ok(writer: &mut impl Write, success: ActionSuccess) -> ReadWriteResult<()> {
    write_all(writer, &[0])?;
    write_string(writer, &success.message)?;
    write_string(writer, success.warning.message())?;
    Ok(())
}

pub fn write_error(writer: &mut impl Write, message: &str) -> ReadWriteResult<()> {
    write_all(writer, &[1])?;
    write_string(writer, message)?;
    Ok(())
}

pub fn write_u16(writer: &mut impl Write, value: u16) -> ReadWriteResult<()> {
    write_all(writer, &value.to_be_bytes())?;
    Ok(())
}

pub fn write_bool(writer: &mut impl Write, value: bool) -> ReadWriteResult<()> {
    write_all(writer, if value { &[1] } else { &[0] })?;
    Ok(())
}

fn write_all(writer: &mut impl Write, buf: &[u8]) -> ReadWriteResult<()> {
    writer.write_all(buf)
        .map_err(ReadWriteError::Io)?;
    
    Ok(())
}