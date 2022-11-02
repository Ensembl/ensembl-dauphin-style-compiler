use serde::{Serialize, ser::SerializeSeq};

use crate::{BackendNamespace};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName {
    backend_namespace: BackendNamespace,
    set: String,
    name: String,
    version: usize
}

impl ProgramName {
    pub fn new(set: &str, name: &str, version: usize, backend_namespace: &BackendNamespace) -> ProgramName {
        ProgramName { set: set.to_string(), name: name.to_string(), version, backend_namespace: backend_namespace.clone() }
    }

    pub fn indicative_name(&self) -> String { format!("{}::{}::{}",self.set,self.name,self.version) }
    pub fn xxx_backendnamespace(&self) -> &BackendNamespace { &self.backend_namespace }
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
