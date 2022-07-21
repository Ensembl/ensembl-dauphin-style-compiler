use std::{collections::{HashMap, hash_map::DefaultHasher }, sync::Arc, hash::{Hash, Hasher}, iter::FromIterator};
use hashbrown::HashSet;
use peregrine_toolkit::{puzzle::{ StaticValue, StaticAnswer, derived }, eachorevery::eoestruct::{StructTemplate, struct_to_json, StructBuilt}};
use crate::{allotment::core::allotmentname::{AllotmentName, AllotmentNamePart}, shape::metadata::AbstractMetadata};
use serde_json::{ Value as JsonValue, Map as JsonMap };

struct AllotmentData {
    values: HashMap<(AllotmentName,String),Vec<StaticValue<(StructTemplate,Option<String>)>>>,
    reports: Vec<AllotmentName>
}

impl AllotmentData {
    fn new() -> AllotmentData {
        AllotmentData { values: HashMap::new(), reports: vec![] }
    }
}

pub struct LocalAllotmentMetadataBuilder(AllotmentData);

impl LocalAllotmentMetadataBuilder {
    pub(crate) fn new(metadata: &AbstractMetadata) -> LocalAllotmentMetadataBuilder {
        let mut out = LocalAllotmentMetadataBuilder(AllotmentData::new());
        metadata.populate_state(&mut out);
        out
    }

    pub(crate) fn set(&mut self, allotment: &AllotmentName, key: &str, value: StaticValue<StructTemplate>, via_boxes: Option<String>) {
        let value = derived(value, move |x| {
           (x,via_boxes.clone())
        });
        self.0.values.entry((allotment.clone(),key.to_string()))
            .or_insert_with(|| vec![])
            .push(value);
    }

    pub(crate) fn set_reporting(&mut self, allotment: &AllotmentName) {
        self.0.reports.push(allotment.clone());
    }
}

pub struct LocalAllotmentMetadata(Arc<AllotmentData>);

impl LocalAllotmentMetadata {
    pub(crate) fn new(builder: &LocalAllotmentMetadataBuilder) -> LocalAllotmentMetadata {
        LocalAllotmentMetadata(Arc::new(AllotmentData {
            values: builder.0.values.clone(),
            reports: builder.0.reports.clone()
        }))
    }

    pub(crate) fn add(&self, global: &mut GlobalAllotmentMetadataBuilder) {
        global.0.values.extend(self.0.values.iter().map(|(x,y)| {
            (x.clone(),y.clone())
        }));
        global.0.reports.extend(self.0.reports.iter().map(|x| x.clone()));
    }
}

pub struct GlobalAllotmentMetadataBuilder(AllotmentData);

impl GlobalAllotmentMetadataBuilder {
    pub(crate) fn new() -> GlobalAllotmentMetadataBuilder {
        GlobalAllotmentMetadataBuilder(AllotmentData::new())
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

struct MapToReporter(HashSet<AllotmentName>);

impl MapToReporter {
    fn new(reports: &[AllotmentName]) -> MapToReporter {
        MapToReporter(HashSet::from_iter(reports.iter().cloned()))
    }

    fn reporting_allotment(&self, input: &AllotmentName, via_boxes: bool) -> Option<AllotmentName> {
        let mut part = AllotmentNamePart::new(input.clone());
        loop {
            if self.0.contains(&AllotmentName::from_part(&part)) { // TODO inefficient
                return Some(AllotmentName::from_part(&part));
            }
            if let Some((_,new)) = part.pop() {
                part = new;
            } else {
                break;
            }
        }
        if via_boxes {
            None
        } else {
            Some(input.clone())
        }
    }
}

fn merge(input: &[(StructTemplate,Option<String>)]) -> Option<(JsonValue,bool)> {
    let mut via_boxes = false;
    let mut collated = HashMap::new();
    for (template,key) in input {
        if key.is_some() { via_boxes = true; }
        if let Ok(value) = template.build() {
            if let Ok(json) = struct_to_json(&value,None) {
                collated.insert(key.clone(),json);
            }
        }
    }
    if via_boxes {
        if let Some(value) = collated.get(&Some("".to_string())) {
            Some((value.clone(),true))
        } else {
            Some((JsonValue::Array(collated.drain().map(|x| x.1).collect()),true))
        }
    } else {
        collated.get(&None).map(|x| (x.clone(),false))
    }
}

impl GlobalAllotmentMetadata {
    pub(crate) fn new(builder: GlobalAllotmentMetadataBuilder, answer: &mut StaticAnswer) -> GlobalAllotmentMetadata {
        let mapper = MapToReporter::new(&builder.0.reports);
        let mut values = HashMap::new();
        for ((allotment,key),value) in &builder.0.values {
            let input_values = value.iter().map(|x| x.call(answer)).collect::<Vec<_>>();
            if let Some((value,via_boxes)) = merge(&input_values) {
                if let Some(reporting) = mapper.reporting_allotment(allotment,via_boxes) {
                    values.insert((reporting,key.to_string()),value.to_string());
                }
            }
        }
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
