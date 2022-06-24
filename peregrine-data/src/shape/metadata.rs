use std::{sync::Arc, collections::HashMap};

use peregrine_toolkit::{puzzle::constant, eachorevery::{EachOrEvery, eoestruct::StructTemplate}};

use crate::{allotment::{core::allotmentname::AllotmentName, globals::allotmentmetadata::LocalAllotmentMetadataBuilder}, Shape, Patina, LeafRequest};
use super::shape::UnplacedShape;

struct AllotmentMetadataEntry {
    allotment: AllotmentName,
    key: String,
    value: StructTemplate
}

impl AllotmentMetadataEntry {
    fn new(allotment: &AllotmentName, key: &str, value: &StructTemplate) -> AllotmentMetadataEntry {
        AllotmentMetadataEntry {
            allotment: allotment.clone(),
            key: key.to_string(),
            value: value.clone()
        }
    }

    fn add(&self, state: &mut LocalAllotmentMetadataBuilder) {
        state.set(&self.allotment,&self.key,constant(self.value.clone()))
    }
}

pub(crate) struct AbstractMetadataBuilder {
    data: Vec<AllotmentMetadataEntry>
}

fn allotment_and_value<'a>(allotments: &'a EachOrEvery<LeafRequest>, values: &'a EachOrEvery<StructTemplate>) -> Option<impl Iterator<Item=(&'a LeafRequest,&'a StructTemplate)>> {
    let len = if let Some(len) = allotments.len() { len } else { return None };
    if !values.compatible(len) { return None; } // XXX proper error without length match
    let iter = allotments.iter(len).unwrap().zip(values.iter(len).unwrap());
    Some(iter)
}

impl AbstractMetadataBuilder {
    pub(crate) fn new() -> AbstractMetadataBuilder {
        AbstractMetadataBuilder { data: vec![] }
    }

    fn add_shape(&mut self, allotments: &EachOrEvery<LeafRequest>, key: &str, values: &EachOrEvery<StructTemplate>) {
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

fn parse_report_value(input: &str) -> Arc<HashMap<String,String>> {
    let parts = input.split(";").collect::<Vec<_>>();
    let mut out = HashMap::new();
    for item in &parts {
        let (key,value) = if let Some(eq_at) = item.find("=") {
            let (k,v) = item.split_at(eq_at);
            (k,&v[1..])
        } else {
            ("type",*item)
        };
        out.insert(key.to_string(),value.to_string());
    }
    Arc::new(out)
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub(crate) struct MetadataStyle {
    values: Arc<HashMap<String,String>>
}

impl MetadataStyle {
    pub(crate) fn new(spec: &str) -> MetadataStyle {
        let values = parse_report_value(spec);
        MetadataStyle { values }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item=(&String,&String)> {
        self.values.iter()
    }
}
