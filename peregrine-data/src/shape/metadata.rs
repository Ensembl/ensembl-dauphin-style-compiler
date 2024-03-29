use eachorevery::eoestruct::{StructValue};
use peregrine_toolkit::{puzzle::constant};
use crate::{allotment::{core::allotmentname::AllotmentName}, globals::allotmentmetadata::LocalAllotmentMetadataBuilder};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub(crate) struct AllotmentMetadataEntry {
    allotment: AllotmentName,
    key: String,
    id: String,
    value: StructValue
}

impl AllotmentMetadataEntry {
    pub(crate) fn new(allotment: &AllotmentName, key: &str, id: &str, value: &StructValue) -> AllotmentMetadataEntry {
        AllotmentMetadataEntry {
            allotment: allotment.clone(),
            key: key.to_string(),
            id: id.to_string(),
            value: value.clone()
        }
    }

    pub(crate) fn add(&self, state: &mut LocalAllotmentMetadataBuilder) {
        state.set(&self.allotment,&self.key,constant(self.value.clone()),Some(self.id.to_string()))
    }
}
