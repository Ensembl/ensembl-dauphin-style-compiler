use peregrine_data::DataMessage;
use crate::Message;

pub(crate) fn eoe_throw<X>(kind: &str,input: Option<X>) -> Result<X,Message> {
    input.ok_or_else(|| Message::DataError(DataMessage::LengthMismatch(kind.to_string())))
}
