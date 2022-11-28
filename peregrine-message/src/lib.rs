use std::fmt;

pub enum MessageKind {
    Error,
    Interface
}

pub trait PeregrineMessage : Send + Sync {
    fn kind(&self) -> MessageKind;
    fn knock_on(&self) -> bool;
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
