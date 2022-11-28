use peregrine_message::PeregrineMessage;
use serde_derive::Serialize;
use crate::{PeregrineCoreBase, metric::{errormetric::ErrorMetricReport, metricreporter::ClientMetricReport}, request::core::minirequest::MiniRequestVariety};

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
}

impl MiniRequestVariety for MetricReport {
    fn description(&self) -> String { "metric".to_string() }
    fn opcode(&self) -> u8 { 6 }
}
