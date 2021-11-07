use std::fmt;
use crate::Channel;
use peregrine_toolkit::serde::de_seq_next;
use serde::{Deserializer, Serializer, de::{SeqAccess, Visitor}, ser::SerializeSeq};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName(pub Channel,pub String);

impl serde::Serialize for ProgramName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}

impl fmt::Display for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}

struct ProgramNameVisitor;

impl<'de> Visitor<'de> for ProgramNameVisitor {
    type Value = ProgramName;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a program name") }

    fn visit_seq<S>(self, mut seq: S) -> Result<ProgramName,S::Error> where S: SeqAccess<'de> {
        let channel = de_seq_next(&mut seq)?;
        let name = de_seq_next(&mut seq)?;
        Ok(ProgramName(channel,name))
    }
}

impl<'de> serde::Deserialize<'de> for ProgramName {
    fn deserialize<D>(deserializer: D) -> Result<ProgramName, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramNameVisitor)
    }
}
