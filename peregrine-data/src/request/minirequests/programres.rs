use std::fmt;
use serde::{de::Visitor, Deserialize, Deserializer};
use crate::request::core::miniresponse::MiniResponseVariety;

pub struct ProgramRes;

struct ProgramVisitor;

impl<'de> Visitor<'de> for ProgramVisitor {
    type Value = ProgramRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ProgramRes")
    }

    fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>, {
        Ok(ProgramRes)
    }
}

impl<'de> Deserialize<'de> for ProgramRes {
    fn deserialize<D>(deserializer: D) -> Result<ProgramRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramVisitor)
    }
}

impl MiniResponseVariety for ProgramRes {
    fn description(&self) -> &str { "program" }
}
