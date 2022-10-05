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

    pub(crate) fn cdr_priority(&self) -> u8 {
        match self {
            PacketPriority::Batch => 5,
            PacketPriority::RealTime => 3
        }
    }

    pub(crate) fn get_pace(&self) -> &[f64] {
        match self {
            PacketPriority::Batch => &[0.,5000.,10000.,20000.,20000.,20000.],
            PacketPriority::RealTime => &[0.,0.,500.,2000.,3000.,10000.]
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
