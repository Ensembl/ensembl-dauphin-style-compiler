use std::error::Error;
use std::sync::{ Arc, Mutex };
use std::fmt;
use crate::request::channel::Channel;
use crate::panel::Panel;

#[derive(Clone,Debug,Hash)]
pub enum DataMessage {
    BadDauphinProgram(String),
    BadBootstrapCannotStart(Channel,Box<DataMessage>),
    BackendTimeout(Channel),
    PacketError(Channel,String),
    TemporaryBackendFailure(Channel),
    BackendRefused(Channel,String),
    DataHasNoAssociatedStyle(Vec<String>),
    TaskTimedOut(String),
    TaskUnexpectedlyCancelled(String),
    TaskUnexpectedlySuperfluous(String),
    TaskResultMissing(String),
    TaskUnexpectedlyOngoing(String),
    NoPanelProgram(Panel),
    DataMissing(Box<DataMessage>),
    CodeInvariantFailed(String),
    StickAuthorityUnavailable(Box<DataMessage>),
    XXXTmp(String)
}

impl fmt::Display for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DataMessage::BadDauphinProgram(s) => format!("Bad Dauphin Program: {}",s),
            DataMessage::BadBootstrapCannotStart(c,cause) => format!("Bad bootstrap response. Cannot start. channel={}: {}",c,cause),
            DataMessage::BackendTimeout(c) => format!("Timeout on connection to {}",c),
            DataMessage::PacketError(c,s) => format!("Error sending/receiving packet: '{}' channel={}",s,c),
            DataMessage::TemporaryBackendFailure(c) => format!("Temporary backend failure (retrying) channel={}",c.to_string()),
            DataMessage::BackendRefused(c,s) => format!("Backend refused: '{}' channel={}",s,c),
            DataMessage::DataHasNoAssociatedStyle(tags) => 
                format!("Data has no associated style: tags={}",tags.join(",")),
            DataMessage::TaskTimedOut(s) => format!("Task '{}' timed out",s),
            DataMessage::TaskUnexpectedlyCancelled(s) => format!("Task '{}' unexpectedly cancelled",s),
            DataMessage::TaskUnexpectedlySuperfluous(s) => format!("Task '{}' unexpectedly superfluous",s),
            DataMessage::TaskResultMissing(s) => format!("Task '{}' result unexpectedly missing",s),
            DataMessage::TaskUnexpectedlyOngoing(s) => format!("Task '{}' unexpectedly ongoing",s),
            DataMessage::DataMissing(source) => format!("Data missing due to earlier: {}",source),
            DataMessage::NoPanelProgram(p) => format!("Missing panel program: {:?}",p),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::StickAuthorityUnavailable(source) => format!("stick authority unavailable due to earlier: {}",source),
            DataMessage::XXXTmp(s) => format!("temporary error: {}",s)
        };
        write!(f,"{}",s)
    }
}

impl Error for DataMessage {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DataMessage::DataMissing(s) =>Some(s),
            _ => None
        }
    }
}
