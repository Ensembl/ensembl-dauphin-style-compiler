use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::Arc, hash::{Hash, Hasher}, iter::FromIterator};
use peregrine_toolkit::{puzzle::{ StaticValue, StaticAnswer, derived }, eachorevery::eoestruct::{StructTemplate, struct_to_json}};
use crate::{allotment::core::allotmentname::AllotmentName, shape::metadata::AbstractMetadata};
use serde_json::{ Value as JsonValue, Map as JsonMap };

pub struct LocalAllotmentMetadataBuilder(HashMap<(AllotmentName,String),StaticValue<Option<String>>>);

impl LocalAllotmentMetadataBuilder {
    pub(crate) fn new(metadata: &AbstractMetadata) -> LocalAllotmentMetadataBuilder {
        let mut out = LocalAllotmentMetadataBuilder(HashMap::new());
        metadata.populate_state(&mut out);
        out
    }

    pub(crate) fn set(&mut self, allotment: &AllotmentName, key: &str, value: StaticValue<StructTemplate>) {
        let value_str = derived(value, |x| {
            x.build()
                .and_then(|tmpl| struct_to_json(&tmpl,None))
                .map(|json| json.to_string()).ok()
        });
        self.0.insert((allotment.clone(),key.to_string()),value_str);
    }
}

pub struct LocalAllotmentMetadata(Arc<HashMap<(AllotmentName,String),StaticValue<Option<String>>>>);

impl LocalAllotmentMetadata {
    pub(crate) fn new(builder: &LocalAllotmentMetadataBuilder) -> LocalAllotmentMetadata {
        LocalAllotmentMetadata(Arc::new(builder.0.clone()))
    }

    pub(crate) fn add(&self, global: &mut GlobalAllotmentMetadataBuilder) {
        global.0.extend(self.0.iter().map(|(x,y)| {
            (x.clone(),y.clone())
        }));
    }
}

pub struct GlobalAllotmentMetadataBuilder(HashMap<(AllotmentName,String),StaticValue<Option<String>>>);

impl GlobalAllotmentMetadataBuilder {
    pub(crate) fn new() -> GlobalAllotmentMetadataBuilder {
        GlobalAllotmentMetadataBuilder(HashMap::new())
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct GlobalAllotmentMetadata(u64,Arc<HashMap<(AllotmentName,String),String>>);

impl PartialEq for GlobalAllotmentMetadata {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl Eq for GlobalAllotmentMetadata {}

impl Hash for GlobalAllotmentMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state); }
}

impl GlobalAllotmentMetadata {
    pub(crate) fn new(builder: GlobalAllotmentMetadataBuilder, answer: &mut StaticAnswer) -> GlobalAllotmentMetadata {
        let values = builder.0.iter().filter_map(|(k,v)| 
            v.call(answer).map(|value| (k.clone(),value))
        ).collect::<HashMap<_,_>>();
        let mut hash = DefaultHasher::new();
        let mut keys = values.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for key in keys.drain(..) {
            key.hash(&mut hash);
            values.get(&key).as_ref().unwrap().hash(&mut hash);
        }
        GlobalAllotmentMetadata(hash.finish(),Arc::new(values))
    }

    pub(crate) fn summarize(&self) -> Vec<HashMap<String,JsonValue>> {
        let mut out = HashMap::new();
        for ((allotment,key),value) in self.1.iter() {
            let block = out.entry(allotment.clone()).or_insert_with(|| HashMap::new());
            if let Ok(value) = serde_json::from_str(value) {
                block.insert(key.clone(),value);
            }
        }
        out.drain().map(|x| x.1).collect::<Vec<_>>()
    }

    pub fn summarize_json(&self) -> JsonValue {
        let mut summary = self.summarize();
        JsonValue::Array(summary.drain(..).map(|mut x| {
            JsonValue::Object(JsonMap::from_iter(x.drain()))
        }).collect())
    }
}
