use std::io::{Error, Write};
use nom::combinator::map_res;
use super::*;

pub fn read_var_string(input: &[u8]) -> IResult<&[u8], String> {
    map_res(read_var_buffer, |s| String::from_utf8(s.to_vec()))(input)
}

pub fn write_var_string<W: Write, S: AsRef<str>>(buffer: &mut W, input: S) -> Result<(), Error> {
    let bytes = input.as_ref().as_bytes();
    write_var_buffer(buffer, bytes)?;
    Ok(())
}
