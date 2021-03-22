use std::fmt;
use std::hash::Hash;

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

pub trait PeregrineMessage : Send + Sync {
    fn level(&self) -> MessageLevel;
    fn category(&self) -> MessageCategory;
    fn now_unstable(&self) -> bool;
    fn degraded_experience(&self) -> bool;
    fn code(&self) -> (u64,u64);
    fn to_message_string(&self) -> String;
    fn cause_message(&self) -> Option<&(dyn PeregrineMessage + 'static)> { None }
}

impl fmt::Display for dyn PeregrineMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}

impl fmt::Debug for dyn PeregrineMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}
