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
