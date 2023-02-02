use std::fmt;

use peregrine_toolkit::serdetools::st_field;
use serde::{Serialize, ser::SerializeSeq, de::Visitor, Deserialize, Deserializer};

// XXX whole struct can die
#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName {
    pub(crate) eard: eard_interp::ProgramName,
}

impl ProgramName {
    pub fn new(group: &str, name: &str, version: u32) -> ProgramName {
        ProgramName { eard: eard_interp::ProgramName::new(group,name,version) }
    }

    pub(crate) fn to_eard(&self) -> &eard_interp::ProgramName { &self.eard }
    pub fn indicative_name(&self) -> String { format!("{}::{}::{}",self.eard.group,self.eard.name,self.eard.version) }
    pub fn group(&self) -> &str { &self.eard.group }
    pub fn name(&self) -> &str { &self.eard.name }
    pub fn version(&self) -> u32 { self.eard.version }
}

impl Serialize for ProgramName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.eard.group)?;
        seq.serialize_element(&self.eard.name)?;
        seq.serialize_element(&self.eard.version)?;
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
        let group : String = st_field("group",seq.next_element()?)?;
        let name : String = st_field("name",seq.next_element()?)?;
        let version = st_field("version",seq.next_element()?)?;
        let eard = eard_interp::ProgramName::new(&group,&name,version);
        Ok(ProgramName { eard })
    }
}

impl<'de> Deserialize<'de> for ProgramName {
    fn deserialize<D>(deserializer: D) -> Result<ProgramName, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramNameVisitor)
    }
}
