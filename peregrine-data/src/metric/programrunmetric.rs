use crate::metric::metricutil::FactoredValueBuilder;
use serde::Serializer;
use serde_derive::{ Serialize };
use serde::ser::{ SerializeSeq };
use std::mem::replace;

#[cfg_attr(debug_assertions,derive(Debug))]
struct ProgramRunDatapoint {
    name: usize,
    scale: u64,
    warm: bool,
    net_ms: f64,
    took_ms: f64
}

impl serde::Serialize for ProgramRunDatapoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(5))?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.scale)?;
        seq.serialize_element(&self.warm)?;
        seq.serialize_element(&self.net_ms)?;
        seq.serialize_element(&self.took_ms)?;
        seq.end()
    }
}

pub(super) struct ProgramRunMetricBuilder {
    names: FactoredValueBuilder<String>,
    datapoints: Vec<ProgramRunDatapoint>
}

impl ProgramRunMetricBuilder {
    pub(super) fn new() -> ProgramRunMetricBuilder {
        ProgramRunMetricBuilder {
            names: FactoredValueBuilder::new(),
            datapoints: vec![]
        }
    }

    pub(super) fn add(&mut self, name: &str, scale: u64, warm: bool, net_ms: f64, took_ms: f64) {
        let name = self.names.lookup(&name.to_string());
        self.datapoints.push(ProgramRunDatapoint {
            name,
            scale, warm, net_ms, took_ms
        });
    }
}

#[derive(Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct ProgramRunMetricData {
    names: Vec<String>,
    datapoints: Vec<ProgramRunDatapoint>
}

impl ProgramRunMetricData {
    pub(super) fn new(builder: &mut ProgramRunMetricBuilder) -> ProgramRunMetricData {
        let names = builder.names.build();
        let datapoints = replace(&mut builder.datapoints,vec![]);
        ProgramRunMetricData {
            names, datapoints
        }
    }

    pub(super) fn empty(&self) -> bool { self.datapoints.len() == 0 }
}
