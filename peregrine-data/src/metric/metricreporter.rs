use crate::core::channel::Channel;
use crate::core::channel::PacketPriority;
use crate::metric::programrunmetric::ProgramRunMetricBuilder;
use crate::metric::programrunmetric::ProgramRunMetricData;
use crate::metric::datastreammetric::DatastreamMetricValue;
use crate::metric::datastreammetric::DatastreamMetricKey;
use crate::metric::datastreammetric::DatastreamMetricBuilder;
use crate::metric::datastreammetric::DatastreamMetricData;
use crate::request::core::manager::RequestManager;
use crate::request::core::request::RequestVariant;
use crate::request::core::request::RequestType;
use crate::request::messages::metricreq::MetricReport;
use commander::cdr_timer;
use std::sync::Mutex;
use std::sync::Arc;
use crate::{PgCommander, PgCommanderTaskSpec, add_task };
use serde_derive::{ Serialize };

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ClientMetricReport {
    identity: u64,
    datastream: Arc<DatastreamMetricData>,
    programrun: Arc<ProgramRunMetricData>
}

impl ClientMetricReport {
    fn new(identity: u64, datastream_generator: &mut DatastreamMetricBuilder, programrun_generator: &mut ProgramRunMetricBuilder) -> ClientMetricReport {
        ClientMetricReport {
            identity,
            datastream: Arc::new(DatastreamMetricData::new(datastream_generator)),
            programrun: Arc::new(ProgramRunMetricData::new(programrun_generator))
        }
    }

    fn empty(&self) -> bool { self.datastream.empty() && self.programrun.empty() }
}

struct MetricCollectorData {
    datastream: DatastreamMetricBuilder,
    program_run: ProgramRunMetricBuilder,
    manager_and_channel: Option<(RequestManager,Channel)>,
    identity: u64
}

impl MetricCollectorData {
    fn new() -> MetricCollectorData {
        MetricCollectorData {
            datastream: DatastreamMetricBuilder::new(),
            program_run: ProgramRunMetricBuilder::new(),
            manager_and_channel: None,
            identity: 0
        }
    }

    pub fn bootstrap(&mut self, channel: &Channel, identity: u64, manager: &RequestManager) {
        self.identity = identity;
        self.manager_and_channel = Some((manager.clone(),channel.clone()));
    }

    fn send(&mut self) -> Vec<RequestType> {
        let mut out = vec![];
        let report = ClientMetricReport::new(self.identity,&mut self.datastream,&mut self.program_run);
        if !report.empty() {
            out.push(RequestType::new(RequestVariant::Metric(MetricReport::Client(report))));
        }
        out
    }

    fn manager_and_channel(&self) -> Option<(RequestManager,Channel)> { self.manager_and_channel.clone() }
}

#[derive(Clone)]
pub struct MetricCollector {
    data: Arc<Mutex<MetricCollectorData>>,
}

impl MetricCollector {
    async fn run(&mut self) {
        loop {
            let mut manager_and_channel = self.data.lock().unwrap().manager_and_channel();
            if let Some((manager,channel)) = &mut manager_and_channel {
                let mut messages = self.data.lock().unwrap().send();
                for message in messages.drain(..) {
                    manager.execute(channel.clone(),PacketPriority::Batch,message).await.ok(); 
                }
            }
            cdr_timer(60000.).await;
        }
    }

    pub fn new(commander: &PgCommander) -> MetricCollector {
        let out = MetricCollector {
            data: Arc::new(Mutex::new(MetricCollectorData::new())),
        };
        let mut out2 = out.clone();
        add_task(commander,PgCommanderTaskSpec {
            name: "metric-sender".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                out2.run().await;
                Ok(())
            }),
            stats: false
        });
        out
    }

    pub fn bootstrap(&mut self, channel: &Channel, identity: u64, manager: &RequestManager) {
        self.data.lock().unwrap().bootstrap(channel,identity,manager);
    }

    pub fn add_datastream(&self, key: &DatastreamMetricKey, value: &DatastreamMetricValue) {
        self.data.lock().unwrap().datastream.add(key,value);
    }

    #[cfg(debug_assertions)]
    pub fn program_run(&self, _name: &str, _scale: u64, _only_warm: bool, _net_ms: f64, _took_ms: f64) {
    }

    #[cfg(not(debug_assertions))]
    pub fn program_run(&self, name: &str, scale: u64, only_warm: bool, net_ms: f64, took_ms: f64) {
        self.data.lock().unwrap().program_run.add(name,scale,only_warm,net_ms,took_ms);
    }
}
