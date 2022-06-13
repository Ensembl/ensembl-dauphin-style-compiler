use std::sync::Arc;

use peregrine_toolkit::puzzle::constant;

use crate::{allotment::{core::allotmentname::AllotmentName, globals::allotmentmetadata::LocalAllotmentMetadataBuilder}, Shape, Patina, EachOrEvery, LeafRequest};
use super::shape::UnplacedShape;

struct AllotmentMetadataEntry {
    allotment: AllotmentName,
    key: String,
    value: String
}

impl AllotmentMetadataEntry {
    fn new(allotment: &AllotmentName, key: &str, value: &str) -> AllotmentMetadataEntry {
        AllotmentMetadataEntry {
            allotment: allotment.clone(),
            key: key.to_string(),
            value: value.to_string()
        }
    }

    fn add(&self, state: &mut LocalAllotmentMetadataBuilder) {
        state.set(&self.allotment,&self.key,constant(self.value.clone()))
    }
}

pub(crate) struct AbstractMetadataBuilder {
    data: Vec<AllotmentMetadataEntry>
}

fn allotment_and_value<'a>(allotments: &'a EachOrEvery<LeafRequest>, values: &'a EachOrEvery<String>) -> Option<impl Iterator<Item=(&'a LeafRequest,&'a String)>> {
    let len = if let Some(len) = allotments.len() { len } else { return None };
    if !values.compatible(len) { return None; } // XXX proper error without length match
    let iter = allotments.iter(len).unwrap().zip(values.iter(len).unwrap());
    Some(iter)
}

impl AbstractMetadataBuilder {
    pub(crate) fn new() -> AbstractMetadataBuilder {
        AbstractMetadataBuilder { data: vec![] }
    }

    fn add_shape(&mut self, allotments: &EachOrEvery<LeafRequest>, key: &str, values: &EachOrEvery<String>) {
        let iter = allotment_and_value(allotments,values);
        let iter = if let Some(iter) = iter { iter } else { return; };
        for (request,value) in iter {
            self.data.push(AllotmentMetadataEntry::new(request.name(),key,value));
        }
    }

    pub(crate) fn add_shapes(&mut self, shapes: &[UnplacedShape]) {
        for shape in shapes {
            match shape {
                Shape::SpaceBaseRect(shape) => {
                    match shape.patina() {
                        Patina::Metadata(key,values) => {
                            self.add_shape(shape.area().top_left().allotments(),key,values);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }

    pub(crate) fn build(self) -> AbstractMetadata {
        AbstractMetadata {
            data: Arc::new(self.data)
        }
    }
}

#[derive(Clone)]
pub(crate) struct AbstractMetadata {
    data: Arc<Vec<AllotmentMetadataEntry>>
}

impl AbstractMetadata {
    pub(crate) fn populate_state(&self, state: &mut LocalAllotmentMetadataBuilder) {
        for item in self.data.iter() {
            item.add(state);
        }
    }
}
