use std::io::{Error, Write};
use nom::bytes::complete::take;
use super::*;

pub fn read_var_buffer(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (tail, len) = read_var_u64(input)?;
    let (tail, val) = take(len as usize)(tail)?;
    Ok((tail, val))
}

pub fn write_var_buffer<W: Write>(buffer: &mut W, data: &[u8]) -> IResult<(), Error> {
    write_var_u64(buffer, data.len() as u64)?;
    buffer.write_all(data)?;
    Ok(())
}
