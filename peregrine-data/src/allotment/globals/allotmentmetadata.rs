use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::Arc, hash::{Hash, Hasher}};
use peregrine_toolkit::{puzzle::{ StaticValue, StaticAnswer }};

use crate::allotment::core::allotmentname::AllotmentName;

pub struct LocalAllotmentMetadataBuilder(HashMap<(AllotmentName,String),StaticValue<String>>);

impl LocalAllotmentMetadataBuilder {
    pub(crate) fn new() -> LocalAllotmentMetadataBuilder {
        LocalAllotmentMetadataBuilder(HashMap::new())
    }

    pub(crate) fn set(&mut self, allotment: &AllotmentName, key: &str, value: StaticValue<String>) {
        self.0.insert((allotment.clone(),key.to_string()),value);
    }
}

pub struct LocalAllotmentMetadata(Arc<HashMap<(AllotmentName,String),StaticValue<String>>>);

impl LocalAllotmentMetadata {
    pub(crate) fn new(builder: &LocalAllotmentMetadataBuilder) -> LocalAllotmentMetadata {
        LocalAllotmentMetadata(Arc::new(builder.0.clone()))
    }

    pub(crate) fn add(&self, global: &mut GlobalAllotmentMetadataBuilder) {
        global.0.extend(self.0.iter().map(|(x,y)| (x.clone(),y.clone())));
    }
}

pub struct GlobalAllotmentMetadataBuilder(HashMap<(AllotmentName,String),StaticValue<String>>);

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
        let values = builder.0.iter().map(|(k,v)| (k.clone(),v.call(answer))).collect::<HashMap<_,_>>();
        let mut hash = DefaultHasher::new();
        let mut keys = values.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for key in keys.drain(..) {
            key.hash(&mut hash);
            values.get(&key).as_ref().unwrap().hash(&mut hash);
        }
        GlobalAllotmentMetadata(hash.finish(),Arc::new(values))
    }

    pub fn summarize(&self) -> Vec<HashMap<String,String>> {
        let mut out = HashMap::new();
        for ((allotment,key),value) in self.1.iter() {
            let block = out.entry(allotment.clone()).or_insert_with(|| HashMap::new());
            block.insert(key.clone(),value.clone());
        }
        out.drain().map(|x| x.1).collect::<Vec<_>>()
    }
}
