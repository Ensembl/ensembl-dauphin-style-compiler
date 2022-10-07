use std::fmt::{ self, Display, Formatter };
use serde_derive::{ Serialize, Deserialize };

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord,Serialize,Deserialize)]
pub struct BackendNamespace(String,String);

impl BackendNamespace {
    pub fn new(scheme: &str, rest: &str) -> BackendNamespace {
        BackendNamespace(scheme.to_string(),rest.to_string())
    }

    pub fn or_missing(channel: &Option<BackendNamespace>) -> BackendNamespace {
        channel.clone().unwrap_or_else(|| BackendNamespace::new("",""))
    }
}

impl Display for BackendNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}:{}",self.0,self.1)
    }
}
