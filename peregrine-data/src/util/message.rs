use std::sync::{ Arc, Mutex };
use std::{ hash::{ Hash, Hasher }, fmt };
use std::collections::hash_map::{ DefaultHasher };
use std::error::Error;
use crate::core::channel::Channel;
use crate::shapeload::programname::ProgramName;
use crate::core::stick::StickId;
use crate::train::model::trainextent::TrainExtent;
use peregrine_message::{ MessageKind, MessageAction, MessageLikelihood, PeregrineMessage };
use peregrine_config::ConfigError;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum DataMessage {
    BadDauphinProgram(String),
    BadBootstrapCannotStart(Channel,Box<DataMessage>),
    BackendTimeout(Channel),
    PacketError(Channel,String),
    TemporaryBackendFailure(Channel),
    FatalBackendFailure(Channel),
    BackendRefused(Channel,String),
    DataHasNoAssociatedStyle(Vec<String>),
    TaskTimedOut(String),
    TaskUnexpectedlyCancelled(String),
    TaskUnexpectedlySuperfluous(String),
    TaskResultMissing(String),
    TaskUnexpectedlyOngoing(String),
    DataMissing(Box<DataMessage>),
    CodeInvariantFailed(String),
    AuthorityUnavailable(Box<DataMessage>),
    NoSuchStick(StickId),
    NoSuchJump(String),
    CarriageUnavailable(Vec<DataMessage>),
    DauphinProgramDidNotLoad(ProgramName),
    DauphinIntegrationError(String),
    DauphinRunError(ProgramName,String),
    DauphinProgramMissing(String),
    DataUnavailable(Channel,Box<DataMessage>),
    TunnelError(Arc<Mutex<dyn PeregrineMessage>>),
    NoSuchAllotment(String),
    AllotmentNotCreated(String),
    ConfigError(ConfigError),
    LengthMismatch(String),
    BadBoxStack(String),
}

impl PeregrineMessage for DataMessage {
    fn kind(&self) -> MessageKind {
        match self {
            _ => MessageKind::Error
        }
    }

    fn action(&self) -> MessageAction {
        match self {
            DataMessage::BadBootstrapCannotStart(_,cause) => cause.action(),
            DataMessage::BackendTimeout(_) => MessageAction::RetrySoon,
            DataMessage::TemporaryBackendFailure(_) => MessageAction::Advisory,
            DataMessage::FatalBackendFailure(_) => MessageAction::RetrySoon,
            DataMessage::TaskTimedOut(_) => MessageAction::YourMistake,
            DataMessage::TunnelError(cause) => cause.lock().unwrap().action(),
            _ => MessageAction::OurMistake
        }
    }

    fn likelihood(&self) -> MessageLikelihood {
        match self {
            DataMessage::TunnelError(e) => e.lock().unwrap().likelihood(),
            DataMessage::BackendTimeout(_) => MessageLikelihood::Inevitable,        
            DataMessage::PacketError(_,_) => MessageLikelihood::Inevitable,
            DataMessage::TemporaryBackendFailure(_) => MessageLikelihood::Inevitable,
            DataMessage::FatalBackendFailure(_) => MessageLikelihood::Inevitable,
            DataMessage::BackendRefused(_,_) => MessageLikelihood::Inevitable,
            DataMessage::TaskUnexpectedlyCancelled(_) => MessageLikelihood::Unlikely,
            DataMessage::TaskUnexpectedlySuperfluous(_) => MessageLikelihood::Inconceivable,
            _ => MessageLikelihood::Quality
        }
    }

    fn code(&self) -> (u64,u64) {
        // Next code is 30; 0 is reserved; 499 is last.
        match self {
            DataMessage::BadDauphinProgram(s) => (1,calculate_hash(s)),
            DataMessage::BadBootstrapCannotStart(_,cause) => (2,calculate_hash(&cause.code())),
            DataMessage::BackendTimeout(c) => (3,calculate_hash(c)),
            DataMessage::PacketError(c,s) => (3,calculate_hash(&(c,s))),
            DataMessage::TemporaryBackendFailure(c) => (4,calculate_hash(c)),
            DataMessage::FatalBackendFailure(c) => (18,calculate_hash(c)),
            DataMessage::BackendRefused(c,s) => (5,calculate_hash(&(c,s))),
            DataMessage::DataHasNoAssociatedStyle(tags) => (6,calculate_hash(tags)),
            DataMessage::TaskTimedOut(s) => (7,calculate_hash(s)),
            DataMessage::TaskUnexpectedlyCancelled(s) => (8,calculate_hash(s)),
            DataMessage::TaskUnexpectedlySuperfluous(s) => (9,calculate_hash(s)),
            DataMessage::TaskResultMissing(s) => (10,calculate_hash(s)),
            DataMessage::TaskUnexpectedlyOngoing(s) => (11,calculate_hash(s)),
            DataMessage::DataMissing(cause) => (13,calculate_hash(&cause.code())),
            DataMessage::CodeInvariantFailed(s) => (15,calculate_hash(s)),
            DataMessage::AuthorityUnavailable(cause) => (16,calculate_hash(&cause.code())),
            DataMessage::NoSuchJump(s) => (12,calculate_hash(s)),
            DataMessage::NoSuchStick(s) => (19,calculate_hash(s)),
            DataMessage::CarriageUnavailable(_) => (20,calculate_hash(&())),
            DataMessage::DauphinProgramDidNotLoad(name) => (21,calculate_hash(name)),
            DataMessage::DauphinIntegrationError(e) => (22,calculate_hash(e)),
            DataMessage::DauphinRunError(p,e) => (23,calculate_hash(&(p,e))),
            DataMessage::DauphinProgramMissing(p) => (24,calculate_hash(p)),
            DataMessage::DataUnavailable(c,e) => (14,calculate_hash(&(c,e.code()))),
            DataMessage::NoSuchAllotment(a) => (25,calculate_hash(a)),
            DataMessage::TunnelError(e) => e.lock().unwrap().code(),
            DataMessage::ConfigError(e) => (17,calculate_hash(e)),
            DataMessage::AllotmentNotCreated(e) => (27,calculate_hash(e)),
            DataMessage::LengthMismatch(e) => (28,calculate_hash(e)),
            DataMessage::BadBoxStack(e) => (29,calculate_hash(e)),
        }
    }

    fn knock_on(&self) -> bool {
        match self {
            DataMessage::DataMissing(_) => true,
            DataMessage::AuthorityUnavailable(_) => true,
            DataMessage::CarriageUnavailable(_) => true,
            _ => false
        }
    }

    #[cfg(debug_assertions)]
    fn to_message_string(&self) -> String {
        match self {
            DataMessage::BadDauphinProgram(s) => format!("Bad Dauphin Program: {}",s),
            DataMessage::BadBootstrapCannotStart(c,cause) => format!("Bad bootstrap response. Cannot start. channel={}: {}",c,cause),
            DataMessage::BackendTimeout(c) => format!("Timeout on connection to {}",c),
            DataMessage::PacketError(c,s) => format!("Error sending/receiving packet: '{}' channel={}",s,c),
            DataMessage::TemporaryBackendFailure(c) => format!("Temporary backend failure (retrying) channel={}",c.to_string()),
            DataMessage::FatalBackendFailure(c) => format!("Fatal backend failure channel={}",c.to_string()),
            DataMessage::BackendRefused(c,s) => format!("Backend refused: '{}' channel={}",s,c),
            DataMessage::DataHasNoAssociatedStyle(tags) => 
                format!("Data has no associated style: tags={}",tags.join(",")),
            DataMessage::TaskTimedOut(s) => format!("Task '{}' timed out",s),
            DataMessage::TaskUnexpectedlyCancelled(s) => format!("Task '{}' unexpectedly cancelled",s),
            DataMessage::TaskUnexpectedlySuperfluous(s) => format!("Task '{}' unexpectedly superfluous",s),
            DataMessage::TaskResultMissing(s) => format!("Task '{}' result unexpectedly missing",s),
            DataMessage::TaskUnexpectedlyOngoing(s) => format!("Task '{}' unexpectedly ongoing",s),
            DataMessage::DataMissing(source) => format!("Data missing due to earlier: {}",source),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::AuthorityUnavailable(source) => format!("stick authority unavailable due to earlier: {}",source),
            DataMessage::NoSuchStick(stick) => format!("no such stick: {}",stick),
            DataMessage::NoSuchJump(jump) => format!("no such jump: {}",jump),
            DataMessage::CarriageUnavailable(causes) =>
                format!("carriage unavilable. causes = [{}]",causes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            DataMessage::DauphinProgramDidNotLoad(name) => format!("dauphin program '{}' did not load",name),
            DataMessage::DauphinIntegrationError(message) => format!("dauphin integration error: {}",message),
            DataMessage::DauphinRunError(program,message) => format!("error running dauphin program '{}': {}",program,message),
            DataMessage::DauphinProgramMissing(program) => format!("dauphin program '{}' missing",program),
            DataMessage::DataUnavailable(channel,e) => format!("data unavialable '{}', channel={}",e.to_string(),channel),
            DataMessage::NoSuchAllotment(allotment) => format!("no such allotment '{}'",allotment),
            DataMessage::AllotmentNotCreated(allotment) => format!("allotment not created '{}'",allotment),
            DataMessage::LengthMismatch(e) => format!("length mismatch: {}",e),
            DataMessage::TunnelError(e) => e.lock().unwrap().to_message_string(),
            DataMessage::ConfigError(e) => match e {
                ConfigError::UnknownConfigKey(k) => format!("unknown config key '{}",k),
                ConfigError::BadConfigValue(k,r) => format!("bad config value for key '{}': {}",k,r),
                ConfigError::UninitialisedKey(k) => format!("uninitialised config key {}",k),    
            },
            DataMessage::BadBoxStack(k) => format!("bad box stack: {}",k)
        }
    }

    #[cfg(not(debug_assertions))]
    fn to_message_string(&self) -> String {
        match self {
            DataMessage::BadDauphinProgram(s) => format!("Bad Dauphin Program: {}",s),
            DataMessage::BadBootstrapCannotStart(c,cause) => format!("Bad bootstrap response. Cannot start. channel={}: {}",c,cause),
            DataMessage::BackendTimeout(c) => format!("Timeout on connection to {}",c),
            DataMessage::PacketError(c,s) => format!("Error sending/receiving packet: '{}' channel={}",s,c),
            DataMessage::TemporaryBackendFailure(c) => format!("Temporary backend failure (retrying) channel={}",c.to_string()),
            DataMessage::FatalBackendFailure(c) => format!("Fatal backend failure channel={}",c.to_string()),
            DataMessage::BackendRefused(c,s) => format!("Backend refused: '{}' channel={}",s,c),
            DataMessage::DataHasNoAssociatedStyle(tags) => 
                format!("Data has no associated style: tags={}",tags.join(",")),
            DataMessage::TaskTimedOut(s) => format!("Task '{}' timed out",s),
            DataMessage::TaskUnexpectedlyCancelled(s) => format!("Task '{}' unexpectedly cancelled",s),
            DataMessage::TaskUnexpectedlySuperfluous(s) => format!("Task '{}' unexpectedly superfluous",s),
            DataMessage::TaskResultMissing(s) => format!("Task '{}' result unexpectedly missing",s),
            DataMessage::TaskUnexpectedlyOngoing(s) => format!("Task '{}' unexpectedly ongoing",s),
            DataMessage::DataMissing(source) => format!("Data missing due to earlier: {}",source),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::AuthorityUnavailable(source) => format!("stick authority unavailable due to earlier: {}",source),
            DataMessage::NoSuchStick(stick) => format!("no such stick: {}",stick),
            DataMessage::NoSuchJump(location) => format!("no such jump: {}",location),
            DataMessage::CarriageUnavailable(causes) =>
                format!("carriage unavilable. causes = [{}]",causes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            DataMessage::DauphinProgramDidNotLoad(name) => format!("dauphin program '{}' did not load",name),
            DataMessage::DauphinIntegrationError(message) => format!("dauphin integration error: {}",message),
            DataMessage::DauphinRunError(program,message) => format!("error running dauphin program '{}': {}",program,message),
            DataMessage::DauphinProgramMissing(program) => format!("dauphin program '{}' missing",program),
            DataMessage::DataUnavailable(channel,e) => format!("data unavialable '{}', channel={}",e.to_string(),channel),
            DataMessage::NoSuchAllotment(allotment) => format!("no such allotment '{}'",allotment),
            DataMessage::AllotmentNotCreated(allotment) => format!("allotment not created '{}'",allotment),
            DataMessage::LengthMismatch(e) => format!("length mismatch: {}",e),
            DataMessage::TunnelError(e) => e.lock().unwrap().to_message_string(),
            DataMessage::ConfigError(e) => match e {
                ConfigError::UnknownConfigKey(k) => format!("unknown config key '{}",k),
                ConfigError::BadConfigValue(k,r) => format!("bad config value for key '{}': {}",k,r),
                ConfigError::UninitialisedKey(k) => format!("uninitialised config key {}",k),    
            },
            DataMessage::BadBoxStack(k) => format!("bad box stack: {}",k)
        }
    }

    fn cause_message(&self) -> Option<&(dyn PeregrineMessage + 'static)> {
        self.cause().map(|x| x as &dyn PeregrineMessage)
    }
}

impl DataMessage {
    fn cause(&self) -> Option<&DataMessage> {
        match self {
            DataMessage::DataMissing(s) => Some(s),
            DataMessage::CarriageUnavailable(causes) => Some(&causes[0]),
            _ => None
        }
    }
}

impl fmt::Display for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}

impl Error for DataMessage {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause().map(|x| x as &dyn Error)
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Debug for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }    
}