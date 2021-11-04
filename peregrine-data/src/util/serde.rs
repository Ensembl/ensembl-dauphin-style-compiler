use crate::DataMessage;

pub(crate) fn ser_wrap<T,E>(value: Result<T,E>) -> Result<T,DataMessage> where E: ToString {
    value.map_err(|e| DataMessage::CodeInvariantFailed(format!("cannot serialzie: {}",e.to_string())))
}
