use crate::request::channel::Channel;

pub enum DataMessage {
    BadDauphinProgram(String),
    BadBootstrapCannotStart(Channel),
    BackendTimeout(Channel),
    PacketSendingError(Channel,String),
    TemporaryBackendFailure(Channel),
    GeneralFailure(Channel,String)
}

impl ToString for DataMessage {
    fn to_string(&self) -> String {
        match self {
            DataMessage::BadDauphinProgram(s) => format!("Bad Dauphin Program: {}",s),
            DataMessage::BadBootstrapCannotStart(c) => format!("Bad bootstrap response. Cannot start. channel={}",c),
            DataMessage::BackendTimeout(c) => format!("Timeout on connection to {}",c),
            DataMessage::PacketSendingError(c,s) => format!("Error sending packet: '{}' channel={}",s,c),
            DataMessage::TemporaryBackendFailure(c) => format!("Temporary backend failure (retrying) channel={}",c.to_string()),
            DataMessage::GeneralFailure(c,s) => format!("General failure: '{}' channel={}",s,c)
        }
    }
}