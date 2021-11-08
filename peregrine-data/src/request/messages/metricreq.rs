use peregrine_message::PeregrineMessage;
use serde_derive::Serialize;

use crate::{PeregrineCoreBase, metric::{errormetric::ErrorMetricReport, metricreporter::ClientMetricReport}};

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
