use peregrine_message::PeregrineMessage;
use serde_derive::{ Serialize };

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ErrorMetricReport {
    identity: u64,
    text: String,
    major: u64,
    minor: u64
}

impl ErrorMetricReport{
    pub(crate) fn new(identity: u64, message: &(dyn PeregrineMessage + 'static)) -> ErrorMetricReport {
        ErrorMetricReport {
            identity,
            text: message.to_string(),
            major: message.code().0,
            minor: message.code().1
        }
    }
}
