use std::io::Read;

use crate::core::{ActionPerformError, ActionResult, ActionSuccess};
use crate::core::Warning;
use crate::util::ReadWriteResult;
use crate::util::read_write_error::ReadWriteError;

pub fn read_string_vec(reader: &mut impl Read) -> Result<Vec<String>, ReadWriteError> {
    let size = read_size(reader)?;

    if size == 0 {
        return Err(ReadWriteError::EmptyPackage);
    }

    let mut vec = Vec::new();
    vec.reserve_exact(size);

    for _ in 0..size {
        vec.push(read_string(reader)?);
    }

    Ok(vec)
}

pub fn read_string(reader: &mut impl Read) -> ReadWriteResult<String> {
    let len = read_size(reader)?;

    let mut buf = vec![0u8; len];
    read_exact(reader, &mut buf)?;
    Ok(String::from_utf8(buf).map_err(ReadWriteError::FromUtf8)?)
}


pub fn read_response(reader: &mut impl Read) -> ReadWriteResult<ActionResult> {
    let enum_tag = read_u8(reader)?;

    match enum_tag {
        0 => Ok(ActionResult::Ok(ActionSuccess {
            message: read_string(reader)?,
            warning: Warning::from(read_string(reader)?),
        })),

        1 => Ok(ActionResult::Err(ActionPerformError::new(read_string(reader)?))),

        _ => Err(ReadWriteError::InvalidResponse(format!(
            "Invalid enum tag in daemon response: {enum_tag:#04x}. Expected 0x00 or 0x01"
        ))),
    }
}

pub fn read_u16(reader: &mut impl Read) -> ReadWriteResult<u16> {
    let mut buf = [0u8; 2];
    read_exact(reader, &mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_bool(reader: &mut impl Read) -> ReadWriteResult<bool> {
    Ok(read_u8(reader)? != 0)
}

fn read_u8(reader: &mut impl Read) -> ReadWriteResult<u8> {
    let mut buf = [0u8; 1];
    read_exact(reader, &mut buf)?;
    Ok(buf[0])
}

fn read_size(reader: &mut impl Read) -> ReadWriteResult<usize> {
    let mut buf = [0u8; 8];
    read_exact(reader, &mut buf)?;
    Ok(u64::from_be_bytes(buf) as usize)
}

fn read_exact(reader: &mut impl Read, buf: &mut [u8]) -> ReadWriteResult<()> {
    reader.read_exact(buf).map_err(ReadWriteError::Io)?;
    Ok(())
}