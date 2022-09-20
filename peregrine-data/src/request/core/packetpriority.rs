use std::fmt::{ self, Display, Formatter };
use serde_derive::{ Serialize };

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,Serialize)]
pub enum PacketPriority {
    RealTime,
    Batch
}

impl PacketPriority {
    pub fn index(&self) -> usize {
        match self {
            PacketPriority::RealTime => 0,
            PacketPriority::Batch => 1
        }
    }
}

impl Display for PacketPriority {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PacketPriority::RealTime => write!(f,"real-time"),
            PacketPriority::Batch => write!(f,"batch")
        }
    }
}
