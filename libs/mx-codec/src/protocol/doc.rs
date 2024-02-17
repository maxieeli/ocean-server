use super::*;

// doc sync message
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum DocMessage {
    // state vector
    // TODO: temporarily skipped in the test, because yrs decoding needs to ensure that the update in step1
    // is the correct state vector binary and any data can be included in our implementation
    // (we will ensure the correctness of encoding and decoding in the subsequent decoding process)
    #[cfg_attr(test, proptest(skip))]
    Step1(Vec<u8>),
    // update
    Step2(Vec<u8>),
    // update
    Update(Vec<u8>),
}

const DOC_MESSAGE_STEP1: u64 = 0;
const DOC_MESSAGE_STEP2: u64 = 1;
const DOC_MESSAGE_UPDATE: u64 = 2;

pub fn read_doc_message(input: &[u8]) -> IResult<&[u8], DocMessage> {
    let (tail, step) = read_var_u64(input)?;
    match step {
        DOC_MESSAGE_STEP1 => {
            let (tail, sv) = read_var_buffer(tail)?;
            // TODO: decode state vector
            Ok((tail, DocMessage::Step1(sv.into())))
        }
        DOC_MESSAGE_STEP2 => {
            let (tail, update) = read_var_buffer(tail)?;
            // TODO: decode update
            Ok((tail, DocMessage::Step2(update.into())))
        }
        DOC_MESSAGE_UPDATE => {
            let (tail, update) = read_var_buffer(tail)?;
            // TODO: decode update
            Ok((tail, DocMessage::Update(update.into())))
        }
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Tag))),
    }
}

pub fn write_doc_message<W: Write>(buffer: &mut W, msg: &DocMessage) -> Result<(), IoError> {
    match msg {
        DocMessage::Step1(sv) => {
            write_var_u64(buffer, DOC_MESSAGE_STEP1)?;
            write_var_buffer(buffer, sv)?;
        }
        DocMessage::Step2(update) => {
            write_var_u64(buffer, DOC_MESSAGE_STEP2)?;
            write_var_buffer(buffer, update)?;
        }
        DocMessage::Update(update) => {
            write_var_u64(buffer, DOC_MESSAGE_UPDATE)?;
            write_var_buffer(buffer, update)?;
        }
    }
    Ok(())
}
