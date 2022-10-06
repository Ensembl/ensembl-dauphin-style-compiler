use crate::BackendNamespace;
use crate::metric::programrunmetric::ProgramRunMetricBuilder;
use crate::metric::programrunmetric::ProgramRunMetricData;
use crate::metric::datastreammetric::DatastreamMetricValue;
use crate::metric::datastreammetric::DatastreamMetricKey;
use crate::metric::datastreammetric::DatastreamMetricBuilder;
use crate::metric::datastreammetric::DatastreamMetricData;
use crate::request::core::manager::RequestManager;
use crate::request::core::request::MiniRequest;
use crate::request::minirequests::metricreq::MetricReport;
use commander::cdr_timer;
use peregrine_toolkit::lock;
use peregrine_toolkit::log_extra;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use std::sync::Mutex;
use std::sync::Arc;
use crate::{PgCommander, PgCommanderTaskSpec, add_task };
use serde_derive::{ Serialize };

use super::generalreporter::GeneralMetricBuilder;
use super::generalreporter::GeneralMetricData;

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ClientMetricReport {
    identity: u64,
    datastream: Arc<DatastreamMetricData>,
    programrun: Arc<ProgramRunMetricData>,
    general: Arc<GeneralMetricData>
}

impl ClientMetricReport {
    fn new(identity: u64, datastream_generator: &mut DatastreamMetricBuilder, programrun_generator: &mut ProgramRunMetricBuilder, general_generator: &mut GeneralMetricBuilder) -> ClientMetricReport {
        ClientMetricReport {
            identity,
            datastream: Arc::new(DatastreamMetricData::new(datastream_generator)),
            programrun: Arc::new(ProgramRunMetricData::new(programrun_generator)),
            general: Arc::new(GeneralMetricData::new(general_generator))
        }
    }

    fn empty(&self) -> bool { self.datastream.empty() && self.programrun.empty() }
}

struct MetricCollectorData {
    datastream: DatastreamMetricBuilder,
    program_run: ProgramRunMetricBuilder,
    general: GeneralMetricBuilder,
    manager_and_channel: Option<(RequestManager,BackendNamespace)>,
    identity: u64
}

impl MetricCollectorData {
    fn new() -> MetricCollectorData {
        MetricCollectorData {
            datastream: DatastreamMetricBuilder::new(),
            program_run: ProgramRunMetricBuilder::new(),
            general: GeneralMetricBuilder::new(),
            manager_and_channel: None,
            identity: 0
        }
    }

    pub fn bootstrap(&mut self, channel: &BackendNamespace, identity: u64, manager: &RequestManager) {
        self.identity = identity;
        self.manager_and_channel = Some((manager.clone(),channel.clone()));
    }

    fn send(&mut self) -> Vec<MiniRequest> {
        let mut out = vec![];
        let report = ClientMetricReport::new(self.identity,&mut self.datastream,&mut self.program_run, &mut self.general);
        if !report.empty() {
            out.push(MiniRequest::Metric(MetricReport::Client(report)));
        }
        out
    }

    fn manager_and_channel(&self) -> Option<(RequestManager,BackendNamespace)> { self.manager_and_channel.clone() }
}

#[derive(Clone)]
pub struct MetricCollector {
    data: Arc<Mutex<MetricCollectorData>>,
}

impl MetricCollector {
    async fn run(&mut self, shutdown: &OneShot) {
        loop {
            let mut manager_and_channel = lock!(self.data).manager_and_channel();
            if let Some((manager,channel)) = &mut manager_and_channel {
                let mut messages = self.data.lock().unwrap().send();
                for message in messages.drain(..) {
                    manager.execute_and_forget(&channel,message);
                }
            }
            for _ in 0..60 {
                cdr_timer(1000.).await;
                if shutdown.poll() { break; }
            }
            if shutdown.poll() { break; }
        }
        log_extra!("metric reporter quitting");
    }

    pub fn new(commander: &PgCommander, shutdown: &OneShot) -> MetricCollector {
        let out = MetricCollector {
            data: Arc::new(Mutex::new(MetricCollectorData::new())),
        };
        let shutdown = shutdown.clone();
        let mut out2 = out.clone();
        add_task(commander,PgCommanderTaskSpec {
            name: "metric-sender".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                out2.run(&shutdown).await;
                Ok(())
            }),
            stats: false
        });
        out
    }

    pub fn bootstrap(&mut self, channel: &BackendNamespace, identity: u64, manager: &RequestManager) {
       lock!(self.data).bootstrap(channel,identity,manager);
    }

    pub fn add_datastream(&self, key: &DatastreamMetricKey, value: &DatastreamMetricValue) {
        lock!(self.data).datastream.add(key,value);
    }

    pub fn add_general(&self, name: &str, tags: &[(String,String)], values: &[(String,f64)]) {
        lock!(self.data).general.add(name,tags,values);
    }

    #[cfg(debug_assertions)]
    pub fn program_run(&self, _name: &str, _scale: u64, _only_warm: bool, _net_ms: f64, _took_ms: f64) {
    }

    #[cfg(not(debug_assertions))]
    pub fn program_run(&self, name: &str, scale: u64, only_warm: bool, net_ms: f64, took_ms: f64) {
        self.data.lock().unwrap().program_run.add(name,scale,only_warm,net_ms,took_ms);
    }
}
