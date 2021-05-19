use std::sync::{ Arc, Mutex };
use std::{ hash::{ Hash, Hasher }, fmt };
use std::collections::hash_map::{ DefaultHasher };
use std::error::Error;
use crate::{ConfigKey, request::channel::Channel};
use crate::lane::ShapeRequest;
use crate::lane::programname::ProgramName;
use crate::core::stick::StickId;
use crate::train::CarriageId;
use peregrine_message::{ MessageLevel, MessageCategory, PeregrineMessage };
use peregrine_config::ConfigError;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Clone,Debug)]
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
    NoLaneProgram(ShapeRequest),
    DataMissing(Box<DataMessage>),
    CodeInvariantFailed(String),
    StickAuthorityUnavailable(Box<DataMessage>),
    NoSuchStick(StickId),
    CarriageUnavailable(CarriageId,Vec<DataMessage>),
    DauphinProgramDidNotLoad(ProgramName),
    DauphinIntegrationError(String),
    DauphinRunError(ProgramName,String),
    DauphinProgramMissing(String),
    DataUnavailable(Channel,Box<DataMessage>),
    TunnelError(Arc<Mutex<dyn PeregrineMessage>>),
    NoSuchAllotment(String),
    ConfigError(ConfigError)
}

impl PeregrineMessage for DataMessage {
    fn level(&self) -> MessageLevel {
        match self {
            DataMessage::TemporaryBackendFailure(_) => MessageLevel::Warn,
            _ => MessageLevel::Error
        }
    }

    fn category(&self) -> MessageCategory {
        match self {
            DataMessage::BadDauphinProgram(_) => MessageCategory::BadData,
            DataMessage::BadBootstrapCannotStart(_,cause) => cause.category(),
            DataMessage::BackendTimeout(_) => MessageCategory::BadInfrastructure,
            DataMessage::PacketError(_,_) => MessageCategory::BadBackend,
            DataMessage::TemporaryBackendFailure(_) => MessageCategory::BadInfrastructure,
            DataMessage::FatalBackendFailure(_) => MessageCategory::BadInfrastructure,
            DataMessage::BackendRefused(_,_) => MessageCategory::BadBackend,
            DataMessage::DataHasNoAssociatedStyle(_) => MessageCategory::BadData,
            DataMessage::TaskTimedOut(_) => MessageCategory::Unknown,
            DataMessage::TaskUnexpectedlyCancelled(_) => MessageCategory::BadCode,
            DataMessage::TaskUnexpectedlySuperfluous(_) => MessageCategory::BadCode,
            DataMessage::TaskResultMissing(_) => MessageCategory::BadCode,
            DataMessage::TaskUnexpectedlyOngoing(_) => MessageCategory::BadCode,
            DataMessage::DataMissing(_) => MessageCategory::Unknown,
            DataMessage::NoLaneProgram(_) => MessageCategory::BadData,
            DataMessage::CodeInvariantFailed(_) => MessageCategory::BadCode,
            DataMessage::StickAuthorityUnavailable(cause) => cause.category(),
            DataMessage::NoSuchStick(_) => MessageCategory::BadFrontend,
            DataMessage::CarriageUnavailable(_,_) => MessageCategory::BadInfrastructure,
            DataMessage::DauphinProgramDidNotLoad(_) => MessageCategory::BadBackend,
            DataMessage::DauphinIntegrationError(_) => MessageCategory::BadCode,
            DataMessage::DauphinRunError(_,_) => MessageCategory::BadData,
            DataMessage::DauphinProgramMissing(_) => MessageCategory::BadData,
            DataMessage::DataUnavailable(_,_) => MessageCategory::BadInfrastructure,
            DataMessage::TunnelError(_) => MessageCategory::BadInfrastructure,
            DataMessage::NoSuchAllotment(_) => MessageCategory::BadData,
            DataMessage::ConfigError(_) => MessageCategory::BadFrontend,
        }
    }

    fn now_unstable(&self) -> bool {
        match self {
            DataMessage::BadBootstrapCannotStart(_,_) => true,
            DataMessage::TaskTimedOut(_) => true,
            DataMessage::TaskUnexpectedlyCancelled(_) => true,
            DataMessage::TaskUnexpectedlySuperfluous(_) => true,
            DataMessage::TaskResultMissing(_) => true,
            DataMessage::TaskUnexpectedlyOngoing(_) => true,
            DataMessage::DauphinProgramDidNotLoad(_) => true,
            DataMessage::DauphinIntegrationError(_) => true,
            DataMessage::DauphinProgramMissing(_) => true,
            DataMessage::NoSuchAllotment(_) => true,
            DataMessage::TunnelError(e) => e.lock().unwrap().now_unstable(),
            DataMessage::ConfigError(_) => true,
            _ => false
        }
    }

    fn degraded_experience(&self) -> bool {
        if self.now_unstable() { return true; }
        match self {
            DataMessage::TemporaryBackendFailure(_) => false,
            DataMessage::TunnelError(e) => e.lock().unwrap().degraded_experience(),
            _ => true
        }
    }

    fn code(&self) -> (u64,u64) {
        // Next code is 27; 25 is unused; 499 is last.
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
            DataMessage::NoLaneProgram(p) => (12,calculate_hash(p)),
            DataMessage::DataMissing(cause) => (13,calculate_hash(&cause.code())),
            DataMessage::CodeInvariantFailed(s) => (15,calculate_hash(s)),
            DataMessage::StickAuthorityUnavailable(cause) => (16,calculate_hash(&cause.code())),
            DataMessage::NoSuchStick(s) => (19,calculate_hash(s)),
            DataMessage::CarriageUnavailable(c,_) => (20,calculate_hash(c)),
            DataMessage::DauphinProgramDidNotLoad(name) => (21,calculate_hash(name)),
            DataMessage::DauphinIntegrationError(e) => (22,calculate_hash(e)),
            DataMessage::DauphinRunError(p,e) => (23,calculate_hash(&(p,e))),
            DataMessage::DauphinProgramMissing(p) => (24,calculate_hash(p)),
            DataMessage::DataUnavailable(c,e) => (14,calculate_hash(&(c,e.code()))),
            DataMessage::NoSuchAllotment(a) => (0,calculate_hash(a)),
            DataMessage::TunnelError(e) => e.lock().unwrap().code(),
            DataMessage::ConfigError(e) => (17,calculate_hash(e)),
        }
    }

    fn knock_on(&self) -> bool {
        match self {
            DataMessage::DataMissing(_) => true,
            DataMessage::StickAuthorityUnavailable(_) => true,
            DataMessage::CarriageUnavailable(_,_) => true,
            _ => false
        }
    }

    fn to_message_string(&self) -> String {
        match self {
            DataMessage::BadDauphinProgram(s) => format!("Bad Dauphin Program: {}",s),
            DataMessage::BadBootstrapCannotStart(c,cause) => format!("Bad bootstrap response. Cannot start. channel={}: {}",c,cause),
            DataMessage::BackendTimeout(c) => format!("Timeout on connection to {}",c),
            DataMessage::PacketError(c,s) => format!("Error sending/receiving packet: '{}' channel={}",s,c),
            DataMessage::TemporaryBackendFailure(c) => format!("Temporary backend failure (retrying) channel={}",c.to_string()),
            DataMessage::FatalBackendFailure(c) => format!("Fatal backend failure (retrying) channel={}",c.to_string()),
            DataMessage::BackendRefused(c,s) => format!("Backend refused: '{}' channel={}",s,c),
            DataMessage::DataHasNoAssociatedStyle(tags) => 
                format!("Data has no associated style: tags={}",tags.join(",")),
            DataMessage::TaskTimedOut(s) => format!("Task '{}' timed out",s),
            DataMessage::TaskUnexpectedlyCancelled(s) => format!("Task '{}' unexpectedly cancelled",s),
            DataMessage::TaskUnexpectedlySuperfluous(s) => format!("Task '{}' unexpectedly superfluous",s),
            DataMessage::TaskResultMissing(s) => format!("Task '{}' result unexpectedly missing",s),
            DataMessage::TaskUnexpectedlyOngoing(s) => format!("Task '{}' unexpectedly ongoing",s),
            DataMessage::DataMissing(source) => format!("Data missing due to earlier: {}",source),
            DataMessage::NoLaneProgram(p) => format!("Missing lane program: {:?}",p),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::StickAuthorityUnavailable(source) => format!("stick authority unavailable due to earlier: {}",source),
            DataMessage::NoSuchStick(stick) => format!("no such stick: {}",stick),
            DataMessage::CarriageUnavailable(id,causes) =>
                format!("carriage {:?} unavilable. causes = [{}]",id,causes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            DataMessage::DauphinProgramDidNotLoad(name) => format!("dauphin program '{}' did not load",name),
            DataMessage::DauphinIntegrationError(message) => format!("dauphin integration error: {}",message),
            DataMessage::DauphinRunError(program,message) => format!("error running dauphin program '{}': {}",program,message),
            DataMessage::DauphinProgramMissing(program) => format!("dauphin program '{}' missing",program),
            DataMessage::DataUnavailable(channel,e) => format!("data unavialable '{}', channel={}",e.to_string(),channel),
            DataMessage::NoSuchAllotment(allotment) => format!("no such allotment '{}'",allotment),
            DataMessage::TunnelError(e) => e.lock().unwrap().to_message_string(),
            DataMessage::ConfigError(e) => match e {
                ConfigError::UnknownConfigKey(k) => format!("unknown config key '{}",k),
                ConfigError::BadConfigValue(k,r) => format!("bad config value for key '{}': {}",k,r),
                ConfigError::UninitialisedKey(k) => format!("uninitialised config key {}",k),    
            }
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
            DataMessage::CarriageUnavailable(_,causes) => Some(&causes[0]),
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
