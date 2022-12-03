use std::{sync::Arc, collections::HashMap};
use peregrine_toolkit::{puzzle::constant, eachorevery::{EachOrEvery, eoestruct::StructTemplate}};
use crate::{allotment::{core::allotmentname::AllotmentName}, Shape, Patina, LeafRequest, globals::allotmentmetadata::LocalAllotmentMetadataBuilder};

struct AllotmentMetadataEntry {
    allotment: AllotmentName,
    key: String,
    id: String,
    value: StructTemplate
}

impl AllotmentMetadataEntry {
    fn new(allotment: &AllotmentName, key: &str, id: &str, value: &StructTemplate) -> AllotmentMetadataEntry {
        AllotmentMetadataEntry {
            allotment: allotment.clone(),
            key: key.to_string(),
            id: id.to_string(),
            value: value.clone()
        }
    }

    fn add(&self, state: &mut LocalAllotmentMetadataBuilder) {
        state.set(&self.allotment,&self.key,constant(self.value.clone()),Some(self.id.to_string()))
    }
}

pub(crate) struct AbstractMetadataBuilder {
    data: Vec<AllotmentMetadataEntry>
}

fn allotment_and_value<'a>(allotments: &'a EachOrEvery<LeafRequest>, values: &'a EachOrEvery<(String,StructTemplate)>) -> Option<impl Iterator<Item=(&'a LeafRequest,&'a (String,StructTemplate))>> {
    let len = if let Some(len) = values.len() { len } else { return None };
    if !allotments.compatible(len) { return None; } // XXX proper error without length match
    let iter = allotments.iter(len).unwrap().zip(values.iter(len).unwrap());
    Some(iter)
}

impl AbstractMetadataBuilder {
    pub(crate) fn new() -> AbstractMetadataBuilder {
        AbstractMetadataBuilder { data: vec![] }
    }

    fn add_shape(&mut self, allotments: &EachOrEvery<LeafRequest>, key: &str, values: &EachOrEvery<(String,StructTemplate)>) {
        let iter = allotment_and_value(allotments,values);
        let iter = if let Some(iter) = iter { iter } else { return; };
        for (request,value) in iter {
            self.data.push(AllotmentMetadataEntry::new(request.name(),key,&value.0,&value.1));
        }
    }

    pub(crate) fn add_shapes(&mut self, shapes: &[Shape<LeafRequest>]) {
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
