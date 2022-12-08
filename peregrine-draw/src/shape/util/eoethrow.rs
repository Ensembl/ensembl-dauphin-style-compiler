use peregrine_data::DataMessage;
use peregrine_toolkit::error::Error;
use crate::Message;

pub(crate) fn eoe_throw<X>(kind: &str,input: Option<X>) -> Result<X,Message> {
    input.ok_or_else(|| Message::DataError(DataMessage::LengthMismatch(kind.to_string())))
}

pub(crate) fn eoe_throw2<X>(kind: &str,input: Option<X>) -> Result<X,Error> {
    input.ok_or_else(|| Error::fatal(&format!("length mismatch {}",kind)))
}
