use peregrine_message::PeregrineMessage;
use serde_derive::Serialize;
use crate::{PeregrineCoreBase, metric::{errormetric::ErrorMetricReport, metricreporter::ClientMetricReport}, request::core::request::MiniRequestVariety};
use serde_cbor::Value as CborValue;

#[derive(Clone,Serialize)]
#[serde(tag = "type")]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum MetricReport {
    Client(ClientMetricReport),
    Error(ErrorMetricReport)
}

impl MetricReport {
    pub fn new_from_error_message(base: &PeregrineCoreBase, message: &(dyn PeregrineMessage + 'static)) -> MetricReport {
        let identity = *base.identity.lock().unwrap();
        MetricReport::Error(ErrorMetricReport::new(identity,message))
    }

    pub fn encode(&self) -> CborValue {
        let xxx = serde_cbor::to_vec(self).ok().unwrap();
        serde_cbor::from_slice(&xxx).ok().unwrap()
    }
}

impl MiniRequestVariety for MetricReport {
    fn description(&self) -> String { "metric".to_string() }
}
