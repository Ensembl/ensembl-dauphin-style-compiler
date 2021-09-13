use crate::metric::metricutil::FactoredValueBuilder;
use serde::Serializer;
use crate::PacketPriority;
use serde_derive::{ Serialize };
use serde::ser::{ SerializeSeq };
use std::mem::replace;

#[derive(PartialEq,Eq,Hash,Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct DatastreamMetricKey {
    pub name: String,
    pub key: String,
    pub scale: u64,
    pub priority: PacketPriority
}

impl DatastreamMetricKey {
    pub fn new(name: &str, key: &str, scale: u64, priority: PacketPriority) -> DatastreamMetricKey {
        DatastreamMetricKey {
            name: name.to_string(),
            key: key.to_string(),
            scale,
            priority
        }
    }
}

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct DatastreamMetricValue {
    pub num_events: u64,
    pub total_size: usize
}

impl DatastreamMetricValue {
    pub fn empty() -> DatastreamMetricValue {
        DatastreamMetricValue {
            num_events: 0,
            total_size: 0
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct DatastreamDatapoint {
    pub name: usize,
    pub key: usize,
    pub scale: u64,
    pub priority: usize,
    pub num_events: u64,
    pub total_size: usize
}

impl serde::Serialize for DatastreamDatapoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(6))?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.key)?;
        seq.serialize_element(&self.scale)?;
        seq.serialize_element(&self.priority)?;
        seq.serialize_element(&self.num_events)?;
        seq.serialize_element(&self.total_size)?;
        seq.end()
    }
}

pub(super) struct DatastreamMetricBuilder {
    names: FactoredValueBuilder<String>,
    keys: FactoredValueBuilder<String>,
    datapoints: Vec<DatastreamDatapoint>
}

impl DatastreamMetricBuilder {
    pub(super) fn new() -> DatastreamMetricBuilder {
        DatastreamMetricBuilder {
            names: FactoredValueBuilder::new(),
            keys: FactoredValueBuilder::new(),
            datapoints: vec![]
        }
    }

    pub(super) fn add(&mut self, mkey: &DatastreamMetricKey, mvalue: &DatastreamMetricValue) {
        let name = self.names.lookup(&mkey.name);
        let key = self.keys.lookup(&mkey.key);
        self.datapoints.push(DatastreamDatapoint {
            name,
            key,
            scale: mkey.scale,
            priority: mkey.priority.index(),
            num_events: mvalue.num_events,
            total_size: mvalue.total_size
        })
    }
}

#[derive(Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct DatastreamMetricData {
    names: Vec<String>,
    keys: Vec<String>,
    datapoints: Vec<DatastreamDatapoint>
}

impl DatastreamMetricData {
    pub(super) fn new(builder: &mut DatastreamMetricBuilder) -> DatastreamMetricData {
        let names = builder.names.build();
        let keys = builder.keys.build();
        let datapoints = replace(&mut builder.datapoints,vec![]);
        DatastreamMetricData {
            names,
            keys,
            datapoints
        }
    }

    pub(super) fn empty(&self) -> bool { self.datapoints.len() == 0 }
}
