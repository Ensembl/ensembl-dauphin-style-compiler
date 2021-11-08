use std::fmt;

use serde::{Deserialize, Deserializer, de::{SeqAccess, Visitor}};

pub struct ProgramCommandResponse {}

struct ProgramVisitor;

impl<'de> Visitor<'de> for ProgramVisitor {
    type Value = ProgramCommandResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a program response") }

    fn visit_seq<S>(self, _seq: S) -> Result<ProgramCommandResponse,S::Error> where S: SeqAccess<'de> {
        Ok(ProgramCommandResponse{})
    }
}

impl<'de> Deserialize<'de> for ProgramCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<ProgramCommandResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramVisitor)
    }
}
