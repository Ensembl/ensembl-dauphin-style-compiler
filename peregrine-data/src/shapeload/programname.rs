use serde::{Serialize, ser::SerializeSeq};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName {
    set: String,
    name: String,
    version: usize
}

impl ProgramName {
    pub fn new(set: &str, name: &str, version: usize) -> ProgramName {
        ProgramName { set: set.to_string(), name: name.to_string(), version }
    }

    pub fn indicative_name(&self) -> String { format!("{}::{}::{}",self.set,self.name,self.version) }
}

impl Serialize for ProgramName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.set)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.version)?;
        seq.end()
    }
}
