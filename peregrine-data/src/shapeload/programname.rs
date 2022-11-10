use std::fmt;

use peregrine_toolkit::serdetools::st_field;
use serde::{Serialize, ser::SerializeSeq, de::Visitor, Deserialize, Deserializer};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName {
    group: String,
    name: String,
    version: usize
}

impl ProgramName {
    pub fn new(group: &str, name: &str, version: usize) -> ProgramName {
        ProgramName { group: group.to_string(), name: name.to_string(), version }
    }

    pub fn indicative_name(&self) -> String { format!("{}::{}::{}",self.group,self.name,self.version) }
}

impl Serialize for ProgramName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.group)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.version)?;
        seq.end()
    }
}

struct ProgramNameVisitor;

impl<'de> Visitor<'de> for ProgramNameVisitor {
    type Value = ProgramName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ProgramName")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let group = st_field("group",seq.next_element()?)?;
        let name = st_field("name",seq.next_element()?)?;
        let version = st_field("version",seq.next_element()?)?;
        Ok(ProgramName { group, name, version })
    }
}

impl<'de> Deserialize<'de> for ProgramName {
    fn deserialize<D>(deserializer: D) -> Result<ProgramName, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramNameVisitor)
    }
}
