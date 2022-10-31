use std::fmt;
use serde_derive::{Serialize, Deserialize};

use crate::{BackendNamespace};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord,Serialize,Deserialize)]
pub struct ProgramName(pub BackendNamespace,pub String);

impl fmt::Display for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord,Serialize,Deserialize)]
pub struct ProgramName2 {
    set: String,
    name: String,
    version: usize
}

impl ProgramName2 {
    pub fn new(set: &str, name: &str, version: usize) -> ProgramName2 {
        ProgramName2 { set: set.to_string(), name: name.to_string(), version }
    }

    pub fn xxx_name(&self) -> &str { &self.name }
}