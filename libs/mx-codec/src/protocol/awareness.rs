use nom::multi::count;
use super::*;

const NULL_STR: &str = "null";

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct AwarenessState {
    #[cfg_attr(test, proptest(strategy = "0..u32::MAX as u64"))]
    pub(crate) clock: u64,
    // content is usually a json
    pub(crate) content: String,
}

impl AwarenessState {
    pub fn new(clock: u64, content: String) -> Self {
        AwarenessState { clock, content }
    }
    pub fn clock(&self) -> u64 {
        self.clock
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_deleted(&self) -> bool {
        self.content == NULL_STR
    }

    pub(crate) fn add_clock(&mut self) {
        self.clock += 1;
    }

    pub(crate) fn set_clock(&mut self, clock: u64) {
        self.clock = clock;
    }

    pub fn set_content(&mut self, content: String) {
        self.add_clock();
        self.content = content;
    }

    pub fn delete(&mut self) {
        self.set_content(NULL_STR.to_string());
    }
}

impl Default for AwarenessState {
    fn default() -> Self {
        AwarenessState {
            clock: 0,
            content: NULL_STR.to_string(),
        }
    }
}

fn read_awareness_state(input: &[u8]) -> IResult<&[u8], (u64, AwarenessState)> {
    let (tail, client_id) = read_var_u64(input)?;
    let (tail, clock) = read_var_u64(tail)?;
    let (tail, content) = read_var_string(tail)?;
    Ok((tail, (client_id, AwarenessState { clock, content })))
}

fn write_awareness_state<W: Write>(
    buffer: &mut W,
    client_id: u64,
    state: &AwarenessState
) -> Result<(), IoError> {
    write_var_u64(buffer, client_id)?;
    write_var_u64(buffer, state.clock)?;
    write_var_string(buffer, state.content.clone())?;
    Ok(())
}

pub type AwarenessStates = HashMap<u64, AwarenessState>;

pub fn read_awareness(input: &[u8]) -> IResult<&[u8], AwarenessStates> {
    let (tail, len) = read_var_u64(input)?;
    let (tail, messages) = count(read_awareness_state, len as usize)(tail)?;
    Ok((tail, messages.into_iter().collect()))
}

pub fn write_awareness<W: Write>(buffer: &mut W, clients: &AwarenessStates) -> Result<(), IoError> {
    write_var_u64(buffer, clients.len() as u64)?;
    for (client_id, state) in clients {
        write_awareness_state(buffer, *client_id, state)?;
    }
    Ok(())
}

// awareness state message
#[derive(Debug, PartialEq)]
pub struct AwarenessMessage {
    clients: AwarenessStates,
}
