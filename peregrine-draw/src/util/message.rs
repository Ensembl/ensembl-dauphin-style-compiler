use std::{fmt::Display, hash::{ Hash, Hasher }, fmt };
use std::collections::hash_map::{ DefaultHasher };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use commander::cdr_identity;
use lazy_static::lazy_static;
use peregrine_data::{ DataMessage, Commander };

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
pub enum MessageLevel {
    Notice,
    Warn,
    Error
}

pub enum MessageCategory {
    BadFrontend,
    BadCode,
    BadData,
    BadBackend,
    BadInfrastructure,
    Unknown
}

pub enum Message {
    DroppedWithoutTidying(String),
    DataError(DataMessage),
    XXXTmp(String)
}

impl Message {
    fn level(&self) -> MessageLevel {
        match self {
            Message::DroppedWithoutTidying(_) => MessageLevel::Warn,
            Message::DataError(d) => {
                match d {
                    DataMessage::TemporaryBackendFailure(_) => MessageLevel::Warn,
                    _ => MessageLevel::Error
                }
            },
            Message::XXXTmp(_) => MessageLevel::Error
        }
    }

    fn category(&self) -> MessageCategory {
        match self {
            Message::DroppedWithoutTidying(_) => MessageCategory::BadCode,
            Message::DataError(d) => {
                match d {
                    DataMessage::BadDauphinProgram(_) => MessageCategory::BadData,
                    DataMessage::BadBootstrapCannotStart(_,cause) => 
                        Message::DataError(cause.as_ref().clone()).category(),
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
                    DataMessage::TaskUnexpectedlyOngoing(s) => MessageCategory::BadCode,
                    DataMessage::DataMissing(_) => MessageCategory::Unknown,
                    DataMessage::NoPanelProgram(_) => MessageCategory::BadData,
                    DataMessage::CodeInvariantFailed(_) => MessageCategory::BadCode,
                    DataMessage::StickAuthorityUnavailable(cause) => 
                        Message::DataError(cause.as_ref().clone()).category(),
                    DataMessage::NoSuchStick(_) => MessageCategory::BadFrontend,
                    DataMessage::XXXTmp(_) => MessageCategory::Unknown,        
                }
            },
            Message::XXXTmp(_) => MessageCategory::Unknown,
        }
    }

    fn now_unstable(&self) -> bool {
        match self {
            Message::DroppedWithoutTidying(_) => false,
            Message::DataError(d) => {
                match d {
                    DataMessage::BadBootstrapCannotStart(_,_) => true,
                    DataMessage::TaskTimedOut(_) => true,
                    DataMessage::TaskUnexpectedlyCancelled(_) => true,
                    DataMessage::TaskUnexpectedlySuperfluous(_) => true,
                    DataMessage::TaskResultMissing(_) => true,
                    DataMessage::TaskUnexpectedlyOngoing(_) => true,
                    _ => false
                }
            },
            Message::XXXTmp(_) => true
        }
    }

    fn degraded_experience(&self) -> bool {
        match self {
            Message::DroppedWithoutTidying(_) => true,
            Message::DataError(d) => {
                match d {
                    DataMessage::TemporaryBackendFailure(c) => false,
                    _ => true
                }
            },
            Message::XXXTmp(_) => true
        }
    }

    fn code(&self) -> (u64,u64) {
        // Next code is 19
        match self {
            Message::DroppedWithoutTidying(s) => (0,calculate_hash(s)),
            Message::DataError(d) => {
                match d {
                    DataMessage::BadDauphinProgram(s) => (1,calculate_hash(s)),
                    DataMessage::BadBootstrapCannotStart(_,cause) => (2,calculate_hash(cause)),
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
                    DataMessage::NoPanelProgram(p) => (12,calculate_hash(p)),
                    DataMessage::DataMissing(cause) =>
                        (13,calculate_hash(&Message::DataError(cause.as_ref().clone()).code())),
                    DataMessage::CodeInvariantFailed(s) => (15,calculate_hash(s)),
                    DataMessage::StickAuthorityUnavailable(cause) => (16,calculate_hash(cause)),
                    DataMessage::NoSuchStick(s) => (19,calculate_hash(s)),
                    DataMessage::XXXTmp(s) => (14,calculate_hash(s)),
                }
            },
            Message::XXXTmp(s) => (17,calculate_hash(s))
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Message::DroppedWithoutTidying(s) => format!("dropped object without tidying: {}",s),
            Message::DataError(d) => d.to_string(),
            Message::XXXTmp(s) => format!("temporary string error: {}",s)
        };
        write!(f,"{}",s)
    }
}

struct MessageCatcher {
    senders: HashMap<Option<u64>,Box<dyn FnMut(Message) + 'static + Send>>,
    default: Option<u64>
}

impl MessageCatcher {
    fn new() -> MessageCatcher {
        MessageCatcher {
            senders: HashMap::new(),
            default: None
        }
    }

    fn default(&mut self, v: u64) { self.default = Some(v); }

    fn add<F>(&mut self, id: Option<u64>, cb: F) where F: FnMut(Message) + 'static + Send {
        self.senders.insert(id,Box::new(cb));
    }

    fn call(&mut self, id : Option<u64>, message: Message) {
        let id = id.or_else(|| self.default);
        if let Some(sender) = self.senders.get_mut(&id) {
            sender(message);
        }
    }
}

lazy_static! {
    static ref message_catcher : Arc<Mutex<MessageCatcher>> = Arc::new(Mutex::new(MessageCatcher::new()));
}    

pub(crate) fn message_register_default(id: u64) {
    message_catcher.lock().unwrap().default(id);
}

pub(crate) fn message_register_callback<F>(id: Option<u64>,cb: F) where F: FnMut(Message) + 'static + Send {
    message_catcher.lock().unwrap().add(id,cb);
}

pub(crate) fn routed_message(id: Option<u64>, message: Message) {
    message_catcher.lock().unwrap().call(id,message);    
}

pub(crate) fn message(message: Message) {
    let id = cdr_identity().map(|x| x.0);
    routed_message(id,message);
}
