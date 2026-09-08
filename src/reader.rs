use std::io::Read;

use crate::daemon::action::{ActionPerformError, ActionResult, ActionSuccess};
use crate::daemon::warning::Warning;
use crate::read_error::ReadError;

pub fn read_string_vec(reader: &mut impl Read) -> Result<Vec<String>, ReadError> {
    let size = read_size(reader)?;

    if size == 0 {
        return Err(ReadError::EmptyPackage);
    }

    let mut vec = Vec::new();
    vec.reserve_exact(size);

    for _ in 0..size {
        vec.push(read_string(reader)?);
    }

    Ok(vec)
}

pub fn read_response(reader: &mut impl Read) -> Result<ActionResult, ReadError> {
    let enum_tag = read_u8(reader)?;

    match enum_tag {
        0 => Ok(ActionResult::Ok(ActionSuccess {
            message: read_string(reader)?,
            warning: Warning::from(read_string(reader)?),
        })),

        1 => Ok(ActionResult::Err(ActionPerformError::new(read_string(
            reader,
        )?))),

        _ => Err(ReadError::invalid_response(format!(
            "Invalid enum tag in daemon response: {enum_tag:#04x}. Expected 0x00 or 0x01"
        ))),
    }
}

fn read_u8(reader: &mut impl Read) -> Result<u8, ReadError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map_err(ReadError::Io)?;
    Ok(buf[0])
}

fn read_size(reader: &mut impl Read) -> Result<usize, ReadError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf).map_err(ReadError::Io)?;
    Ok(u64::from_be_bytes(buf) as usize)
}

fn read_string(reader: &mut impl Read) -> Result<String, ReadError> {
    let len = read_size(reader)?;

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(ReadError::Io)?;
    Ok(String::from_utf8(buf).map_err(ReadError::FromUtf8)?)
}
