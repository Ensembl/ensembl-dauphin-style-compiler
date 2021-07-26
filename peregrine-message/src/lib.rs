use std::fmt;
use std::sync::{ Arc, Mutex };
use commander::SendFusePromise;

pub enum MessageKind {
    Error,
    Interface
}

pub enum MessageAction {
    ImmediateRetry, /* just hit reload */
    RetrySoon,      /* hand on a bit and try again */
    OurMistake,     /* we messed up */
    YourMistake,    /* there's something wrong with your computer */
    Advisory        /* FYI (porbably drop these except in dev builds) */
}

pub enum MessageLikelihood {
    Inevitable,   /* network errors, etc... */
    Quality,      /* we should have done better, but didn't */
    Unlikely,     /* these errors shouldn't happen, but it's clear what did */
    Inconceivable /* how on earth did you get _that_ to happen? */
}

pub trait PeregrineMessage : Send + Sync {
    fn kind(&self) -> MessageKind;
    fn action(&self) -> MessageAction;
    fn likelihood(&self) -> MessageLikelihood;
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
