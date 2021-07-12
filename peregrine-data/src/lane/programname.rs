use std::fmt;
use crate::Channel;
use crate::util::message::DataMessage;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string };

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName(pub Channel,pub String);

impl ProgramName {
    pub(crate) fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![self.0.serialize()?,CborValue::Text(self.1.to_string())]))
    }

    // XXX anyhow!
    pub(crate) fn deserialize(value: &CborValue) -> anyhow::Result<ProgramName> {
        let values = cbor_array(&value,2,false)?;
        Ok(ProgramName(Channel::deserialize(&values[0])?,cbor_string(&values[1])?))
    }
}

impl fmt::Display for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}